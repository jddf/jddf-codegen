.PHONY:
all: \
	examples/user/ts/index.ts \
	examples/message/ts/index.ts \
	examples/gamut/ts/index.ts \
	examples/user/golang/user.go \
	examples/message/golang/message.go \
	examples/gamut/golang/gamut.go

examples/user/ts/index.ts: target/release/jddf-codegen examples/user/user.jddf.json
	target/release/jddf-codegen --ts-out=examples/user/ts -- examples/user/user.jddf.json

examples/message/ts/index.ts: target/release/jddf-codegen examples/message/message.jddf.json
	target/release/jddf-codegen --ts-out=examples/message/ts -- examples/message/message.jddf.json

examples/gamut/ts/index.ts: target/release/jddf-codegen examples/gamut/gamut.jddf.json
	target/release/jddf-codegen --ts-out=examples/gamut/ts -- examples/gamut/gamut.jddf.json

examples/user/golang/user.go: target/release/jddf-codegen examples/user/user.jddf.json
	target/release/jddf-codegen --go-out=examples/user/golang -- examples/user/user.jddf.json

examples/message/golang/message.go: target/release/jddf-codegen examples/message/message.jddf.json
	target/release/jddf-codegen --go-out=examples/message/golang -- examples/message/message.jddf.json

examples/gamut/golang/gamut.go: target/release/jddf-codegen examples/gamut/gamut.jddf.json
	target/release/jddf-codegen --go-out=examples/gamut/golang -- examples/gamut/gamut.jddf.json

target/release/jddf-codegen:
	cargo build --release
