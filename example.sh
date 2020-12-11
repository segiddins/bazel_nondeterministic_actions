#!/usr/bin/env bash

set -euxo pipefail

echo "$(date) $RANDOM" >example/example.txt
bazel build --experimental_execution_log_file=build1.log example:uses_example_file.txt
echo "$(date) $RANDOM" >example/example.txt
bazel build --experimental_execution_log_file=build2.log example:uses_example_file.txt

bazel build @io_bazel//src/tools/execlog:parser
bazel-bin/external/io_bazel/src/tools/execlog/parser --jvm_flag=-Xms6g --jvm_flag=-Xmx26g --log_path build2.log --log_path build1.log --output_path build2.log.txt --output_path build1.log.txt
diff --new-line-format='2|%L' --old-line-format='1|%L' --unchanged-line-format='  %L' build1.log.txt build2.log.txt >build1-2.log.diff.txt || true
bazel build bazel_nondeterministic_actions
bazel-bin/bazel_nondeterministic_actions/bazel_nondeterministic_actions build1-2.log.diff.txt | tee build1-2.log.example.txt
