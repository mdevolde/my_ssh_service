# My SSH service
A Windows SSH service to run an SSH connection in the background.

## Requirements
Have  [everything you need to compile Rust](https://www.rust-lang.org/tools/install).

## How to build and use
To build and run the service, follow these steps (be sure to replace generic values with your own data in the commands):
```powershell
cargo b -r
sc.exe create "mysshservice" binPath="""C:\path\to\your\generated\exe\my_ssh_service.exe"" -N -T -R 9999:localhost:9999 user@ip" obj="DOMAIN\Username" password="UserPassword" depend="nsi" DisplayName="My SSH service" start=auto # needs to be executed by an admin
sc.exe description "mysshservice" "A Windows SSH service to run an SSH connection in the background." # needs to be executed by an admin
sc.exe start mysshservice # needs to be executed by an admin
```
This example runs a reverse port forwarding on port 9999 in the background, but you can pass any arguments you like to SSH.
Obviously, the ssh connection must be able to be established using a key, without the need to enter a passphrase.
This service will start automatically when the computer boots up.

If you want to exceptionally run the service with other arguments (a different port for exemple), you proceed as follows:
```powershell
sc.exe stop "mysshservice" # needs to be executed by an admin
sc.exe start mysshservice -N -T -R 9998:localhost:9998 user@ip # needs to be executed by an admin
```
When the computer boots up, the service will start with the default arguments.

## How to stop the service and check its status
To stop the service, execute this command:
```powershell
sc.exe stop "mysshservice" # needs to be executed by an admin
```
To check the status of the service, you need to run this command:
```powershell
sc.exe query "mysshservice"
```
