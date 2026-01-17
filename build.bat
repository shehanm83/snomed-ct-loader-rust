@echo off
set PROTOC=C:\apps\protobuf-33.4\bin\protoc.exe
cd /d H:\3.0\apps\snomed-ct-loader-rust
cargo build --workspace
