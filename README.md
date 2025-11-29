# TP - Teleport Platform

![alt text](icon.png)

A simple multiplatform app to transfer text from your phone to your PC via a local HTTPS server.

## Features
- **Secure**: Uses self-signed HTTPS certificates.
- **Easy**: Scans a QR code on your PC to open the transfer page on your phone.
- **Automatic**: Copied text on phone is sent to PC and automatically copied to PC clipboard.

## Usage

1. Run the app:
   ```bash
   cargo run
   ```
   Or specify a port:
   ```bash
   cargo run -- --port 8080
   ```
   Or use HTTP instead of HTTPS:
   ```bash
   cargo run -- --http
   ```

2. Allow the app in your firewall if prompted.

3. Scan the QR code displayed in the GUI with your phone.

4. **Accept the self-signed certificate warning on your phone (since it's a local self-signed cert).**

5. Paste text into the text area on your phone and click "Send".

6. The text will appear in the GUI and be copied to your clipboard.

## Requirements
- Rust (latest stable)
- `sudo apt-get install -y cmake nasm mingw-w64` (mingw-w64 and nasm only if installing for windows -> https://github.com/rustls/rustls/issues/1913)

## Note
Ensure your phone and PC are on the same Wi-Fi network.
