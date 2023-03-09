all:
	cd /sigpwny
	cargo build --release --bin car
	cp target/thumbv7em-none-eabihf/release/car ${ELF_PATH}
	/sigpwny/build_bin.py ${ELF_PATH} ${BIN_PATH}
