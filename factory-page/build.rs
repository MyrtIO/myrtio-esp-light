use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/*");
    println!("cargo:rerun-if-changed=index.html");
    println!("cargo:rerun-if-changed=*.ts");

    let output = Command::new("bun")
        .arg("run")
        .arg("build")
        .output()
        .unwrap();
    println!("{}", String::from_utf8(output.stdout).unwrap());
}
