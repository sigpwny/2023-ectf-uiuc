paired_fob:
	cd /sigpwny && cargo build --release --bin fob
	cp /sigpwny/target/thumbv7em-none-eabihf/release/fob ${ELF_PATH}
	arm-none-eabi-objcopy -O binary -R .vector_table ${ELF_PATH} ${BIN_PATH}
	python3 build_fob_eeprom.py ${SECRETS_DIR} ${EEPROM_PATH} ${CAR_ID} ${PAIR_PIN}

unpaired_fob:
	cd /sigpwny && cargo build --release --bin fob
	cp /sigpwny/target/thumbv7em-none-eabihf/release/fob ${ELF_PATH}
	arm-none-eabi-objcopy -O binary -R .vector_table ${ELF_PATH} ${BIN_PATH}
	python3 build_fob_eeprom.py ${SECRETS_DIR} ${EEPROM_PATH}
