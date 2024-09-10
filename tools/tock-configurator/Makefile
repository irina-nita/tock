# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright OxidOS Automotive SRL 2024
#
# Author: Irina Nita <irina.nita@oxidos.io>
# Author: Darius Jipa <darius.jipa@oxidos.io>

PROFRAWS := $(wildcard *.profraw configurator/*.profraw parse/*.profraw parse-macros/*.profraw generator/*.profraw)

# The following variables need to be manually set to get the coverage report.
TOCK_CONFIGURATOR_MAIN := "tock_configurator-6b8185b56fa4ca3b"
TOCK_CONFIGURATOR_LIB := "tock_configurator-7c89136d1e81113c"
TOCK_PARSE := "parse-ac7aafe11dcb2860"
TOCK_PARSE_MACROS := "parse_macros-897eb3cd92805239"
TOCK_GENERATOR := "tock_generator-fc60c49b462360a9"

get_coverage: clean configurator parse parse-macros generator
	make coverage
        
configurator:
	RUSTFLAGS="-C instrument-coverage" cargo test -p "tock-configurator@0.1.0" -- --nocapture

parse:
	RUSTFLAGS="-C instrument-coverage" cargo test -p "parse@0.1.0" -- --nocapture

parse-macros:
	RUSTFLAGS="-C instrument-coverage" cargo test -p "parse-macros@0.1.0" -- --nocapture

generator:
	RUSTFLAGS="-C instrument-coverage" cargo test -p "tock-generator@0.1.0" -- --nocapture

coverage:
	@cargo profdata -- merge -sparse $(PROFRAWS) -o default.profdata
	@cargo cov -- show -show-line-counts-or-regions \
    -output-dir=html -format=html --ignore-filename-regex=‘/.cargo/registry’ \
    --ignore-filename-regex=".*/.cargo/.*"\
    --ignore-filename-regex="rustc/.*"\
    --ignore-filename-regex="generator/.*"\
    --ignore-filename-regex="parse/.*"\
    --ignore-filename-regex="parse-macros/.*"\
    --ignore-filename-regex="main.rs"\
    --ignore-filename-regex="test.rs"\
    --ignore-filename-regex="chips/.*"\
    --instr-profile=default.profdata \
    --object target/debug/deps/$(TOCK_CONFIGURATOR_MAIN) \
    --object target/debug/deps/$(TOCK_CONFIGURATOR_LIB) \
    --object target/debug/deps/$(TOCK_GENERATOR) \
    --object target/debug/deps/$(TOCK_PARSE) \
    --object target/debug/deps/$(TOCK_PARSE_MACROS)
	make clean

clean:
	@rm -f $(PROFRAWS)

.PHONY: get_coverage configurator parse parse-macros generator coverage clean
