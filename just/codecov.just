# Measure test coverage into `lcov.info`
[group('dev')]
coverage:
    cargo llvm-cov --lcov --output-path lcov.info
