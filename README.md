# My SSH service
A Windows SSH service to run an SSH connection in the background.

## Requirements
Have  [everything you need to compile Rust](https://www.rust-lang.org/tools/install).

## How to build and use
To build and run the service, follow these steps (be sure to replace generic values with your own data in the commands):
```powershell
cargo b -r
sc.exe create "mysshservice" binPath="C:\path\to\your\generated\exe\my_ssh_service.exe" obj="DOMAIN\Username" password="UserPassword" depend="nsi" DisplayName="My SSH service" # needs to be executed by an admin
sc start mysshservice -N -T -R 9999:localhost:9999 user@ip # needs to be executed by an admin
```
This example runs a reverse port forwarding on port 9999 in the background, but you can pass any arguments you like to SSH.
Obviously, the ssh connection must be able to be established using a key, without the need to enter a passphrase.

## How to stop the service and check its status
To stop the service, execute this command:
```powershell
sc stop "mysshservice" # needs to be executed by an admin
```
To check the status of the service, you need to run this command:
```powershell
sc query "mysshservice"
```
