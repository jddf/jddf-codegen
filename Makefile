.PHONY: all
all: examples/analytics/ts/index.ts examples/gamut/ts/index.ts

examples/analytics/ts/index.ts: target/release/jddf-codegen examples/analytics/analytics.jddf.json
	target/release/jddf-codegen --ts-out=examples/analytics/ts -- examples/analytics/analytics.jddf.json

examples/gamut/ts/index.ts: target/release/jddf-codegen examples/gamut/gamut.jddf.json
	target/release/jddf-codegen --ts-out=examples/gamut/ts -- examples/gamut/gamut.jddf.json
