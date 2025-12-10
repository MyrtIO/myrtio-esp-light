#!/usr/bin/env python3
"""
Simple espota-style OTA helper for MyrtIO firmware.

This script:
1. Starts an HTTP server to serve the firmware image
2. Sends an OTA invite to the device over TCP
3. Waits for the device to download the firmware

Usage:
    python3 scripts/ota.py --host <device-host> --image <firmware.bin>
    python3 scripts/ota.py --host myrtio-rs1.lan --image target/ota/firmware.bin
"""

import argparse
import hashlib
import http.server
import os
import socket
import socketserver
import sys
import threading
import time
from pathlib import Path


# Default ports
DEFAULT_TCP_PORT = 3232
DEFAULT_HTTP_PORT = 8266
DEFAULT_PATH = "/firmware.bin"

# Timeout for waiting for device response
INVITE_TIMEOUT = 10
# Timeout for the HTTP server (how long to wait for the device to download)
HTTP_SERVER_TIMEOUT = 300  # 5 minutes


class FirmwareHandler(http.server.SimpleHTTPRequestHandler):
    """Custom HTTP handler that serves only the firmware file."""

    firmware_path: str = ""
    firmware_data: bytes = b""
    download_complete = threading.Event()

    def log_message(self, format, *args):
        """Custom log format."""
        print(f"[HTTP] {args[0]}")

    def do_GET(self):
        """Handle GET requests - serve the firmware file."""
        if self.path == DEFAULT_PATH:
            self.send_response(200)
            self.send_header("Content-Type", "application/octet-stream")
            self.send_header("Content-Length", len(self.firmware_data))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(self.firmware_data)
            print(f"[HTTP] Firmware sent ({len(self.firmware_data)} bytes)")
            self.download_complete.set()
        else:
            self.send_error(404, "Not Found")


def get_local_ip():
    """Get the local IP address that can be used to reach the device."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        # Connect to a public IP to determine our local IP
        # This doesn't actually send any data
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
    except Exception:
        ip = "127.0.0.1"
    finally:
        s.close()
    return ip


def compute_md5(data: bytes) -> str:
    """Compute MD5 hash of data."""
    return hashlib.md5(data).hexdigest()


def start_http_server(port: int, firmware_data: bytes) -> socketserver.TCPServer:
    """Start an HTTP server to serve the firmware."""
    FirmwareHandler.firmware_data = firmware_data
    FirmwareHandler.download_complete.clear()

    server = socketserver.TCPServer(("", port), FirmwareHandler)
    server.timeout = 1  # Short timeout to allow checking for shutdown

    thread = threading.Thread(target=lambda: serve_until_complete(server), daemon=True)
    thread.start()

    return server


def serve_until_complete(server: socketserver.TCPServer):
    """Serve HTTP requests until download is complete or timeout."""
    start_time = time.time()
    while not FirmwareHandler.download_complete.is_set():
        server.handle_request()
        if time.time() - start_time > HTTP_SERVER_TIMEOUT:
            print("[HTTP] Server timeout")
            break


def send_ota_invite(host: str, tcp_port: int, http_host: str, http_port: int,
                    path: str, size: int, md5: str) -> bool:
    """Send OTA invite to the device and wait for acknowledgment."""
    print(f"[OTA] Connecting to {host}:{tcp_port}...")

    try:
        # Resolve hostname
        addr_info = socket.getaddrinfo(host, tcp_port, socket.AF_INET, socket.SOCK_STREAM)
        if not addr_info:
            print(f"[OTA] Failed to resolve {host}")
            return False

        addr = addr_info[0][4]
        print(f"[OTA] Resolved to {addr[0]}")

        # Connect to device
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(INVITE_TIMEOUT)
        sock.connect(addr)

        print(f"[OTA] Connected, sending invite...")

        # Build and send invite
        invite = f"HOST={http_host}\nPORT={http_port}\nPATH={path}\nSIZE={size}\nMD5={md5}\n\n"
        sock.sendall(invite.encode())

        print(f"[OTA] Invite sent:")
        print(f"      HOST={http_host}")
        print(f"      PORT={http_port}")
        print(f"      PATH={path}")
        print(f"      SIZE={size}")
        print(f"      MD5={md5}")

        # Wait for ACK
        sock.settimeout(INVITE_TIMEOUT)
        response = sock.recv(64).decode().strip()

        if response == "OK":
            print(f"[OTA] Device acknowledged, starting download...")
            sock.close()
            return True
        else:
            print(f"[OTA] Unexpected response: {response}")
            sock.close()
            return False

    except socket.timeout:
        print(f"[OTA] Connection timeout")
        return False
    except ConnectionRefusedError:
        print(f"[OTA] Connection refused - device may not be running OTA listener")
        return False
    except Exception as e:
        print(f"[OTA] Error: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Send OTA update to MyrtIO device",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    %(prog)s --host myrtio-rs1.lan --image target/ota/firmware.bin
    %(prog)s --host 192.168.1.100 --image firmware.bin --http-port 8080
        """
    )
    parser.add_argument("--host", required=True, help="Device hostname or IP address")
    parser.add_argument("--image", required=True, help="Path to firmware binary")
    parser.add_argument("--tcp-port", type=int, default=DEFAULT_TCP_PORT,
                        help=f"OTA TCP port on device (default: {DEFAULT_TCP_PORT})")
    parser.add_argument("--http-port", type=int, default=DEFAULT_HTTP_PORT,
                        help=f"Local HTTP server port (default: {DEFAULT_HTTP_PORT})")
    parser.add_argument("--path", default=DEFAULT_PATH,
                        help=f"HTTP path for firmware (default: {DEFAULT_PATH})")

    args = parser.parse_args()

    # Read firmware
    image_path = Path(args.image)
    if not image_path.exists():
        print(f"Error: Firmware file not found: {args.image}")
        sys.exit(1)

    firmware_data = image_path.read_bytes()
    firmware_size = len(firmware_data)
    firmware_md5 = compute_md5(firmware_data)

    print(f"[OTA] Firmware: {args.image}")
    print(f"[OTA] Size: {firmware_size} bytes")
    print(f"[OTA] MD5: {firmware_md5}")

    # Determine local IP for HTTP server
    local_ip = get_local_ip()
    print(f"[OTA] Local IP: {local_ip}")

    # Start HTTP server
    print(f"[HTTP] Starting server on port {args.http_port}...")
    server = start_http_server(args.http_port, firmware_data)
    print(f"[HTTP] Server started, serving at http://{local_ip}:{args.http_port}{args.path}")

    # Send OTA invite
    success = send_ota_invite(
        host=args.host,
        tcp_port=args.tcp_port,
        http_host=local_ip,
        http_port=args.http_port,
        path=args.path,
        size=firmware_size,
        md5=firmware_md5
    )

    if not success:
        print("[OTA] Failed to send invite")
        server.shutdown()
        sys.exit(1)

    # Wait for download to complete
    print("[OTA] Waiting for device to download firmware...")
    start_time = time.time()
    while not FirmwareHandler.download_complete.wait(timeout=1):
        elapsed = time.time() - start_time
        if elapsed > HTTP_SERVER_TIMEOUT:
            print("[OTA] Timeout waiting for download")
            server.shutdown()
            sys.exit(1)
        # Print progress dot every 5 seconds
        if int(elapsed) % 5 == 0 and int(elapsed) > 0:
            print(".", end="", flush=True)

    print("\n[OTA] Update complete! Device should reboot shortly.")

    # Give the server a moment to finish
    time.sleep(1)
    server.shutdown()
    sys.exit(0)


if __name__ == "__main__":
    main()

