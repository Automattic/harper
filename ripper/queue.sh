#! /bin/bash

# cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/to_too.txt --clean-file clean.txt --max-mutations 60 --seed to -o to_too.sol 
cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/their_there.txt --clean-file clean.txt --max-mutations 60 -o their_there.sol
cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/there_theyre.txt --clean-file clean.txt --max-mutations 60 -o there_theyre.sol
cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/theyre_their.txt --clean-file clean.txt --max-mutations 60 -o theyre_their.sol
cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/whose_whos.txt --clean-file clean.txt --max-mutations 60 -o whose_whos.sol
cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/whos_whose.txt --clean-file clean.txt --max-mutations 60 -o whos_whose.sol
cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/missing_article.txt --clean-file clean.txt --max-mutations 60 -o missing_article.sol
cargo run --release -- -m 1000000 -g 75 -c 10 --problem-file rules/indp.txt --clean-file clean.txt --max-mutations 60 -o best.sol
cargo run --release -- -m 1600000 -g 200 -c 10 --problem-file rules/missing_noun.txt --clean-file clean.txt --max-mutations 60 -o missing_noun.sol
cargo run --release -- -m 1600000 -g 75 -c 10 --problem-file rules/effect_affect.txt --clean-file clean.txt --max-mutations 60 -o effect_affect.sol
cargo run --release -- -m 1600000 -g 75 -c 10 --problem-file rules/affect_effect.txt --clean-file clean.txt --max-mutations 60 -o affect_effect.sol
cargo run --release -- -m 1600000 -g 75 -c 10 --problem-file rules/missing_to.txt --clean-file clean.txt --max-mutations 60 -o missing_to.sol
