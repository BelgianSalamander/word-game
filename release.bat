rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add x86_64-pc-windows-msvc

cargo build --release --target=x86_64-pc-windows-msvc
xcopy target\x86_64-pc-windows-msvc\release\word-game.exe* release\windows\word-game.exe* /y
xcopy res\ release\windows\res\ /y /e
tar.exe -a -cf release\windows.zip release\windows

@RD /S /Q "release\windows"

find . -type f -name "*.tif" -size -160k -delete

cargo build --release --target=x86_64-apple-darwin
xcopy target\x86_64-apple-darwin\release\word-game.app* release\apple\word-game.app* /y
tar.exe -a -cf release\apple.zip release\apple
@RD /S /Q "release\apple"

find . -type f -name "*.tif" -size -160k -delete


cargo build --release --target=x86_64-unknown-linux-gnu
xcopy target\x86_64-unknown-linux-gnu\release\word-game* release\linux\word-game* /y
tar.exe -a -cf release\apple.zip release\linux
@RD /S /Q "release\linux"

find . -type f -name "*.tif" -size -160k -delete
