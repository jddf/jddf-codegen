.PHONY: all
all: examples/message/ts/index.ts examples/gamut/ts/index.ts

examples/message/ts/index.ts: target/release/jddf-codegen examples/message/message.jddf.json
	target/release/jddf-codegen --ts-out=examples/message/ts -- examples/message/message.jddf.json

examples/gamut/ts/index.ts: target/release/jddf-codegen examples/gamut/gamut.jddf.json
	target/release/jddf-codegen --ts-out=examples/gamut/ts -- examples/gamut/gamut.jddf.json
