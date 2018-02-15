TMP_FILE := $(shell mktemp)

all: use_case_tests

use_case_tests: list_instances_with_filter

list_instances_with_filter: list_instances_with_filter.expected check_environment
	@cd ..; cargo run -- --profile_arn $(AWS_ARN) instances list --filter Na:dns > $(TMP_FILE)
	@diff $< $(TMP_FILE)

check_environment:
	@if test -z "$(AWS_ARN)"; then echo "AWS profile ARN is not set! Set ARN as environment variable 'AWS_ARN' with 'ARN=<arn>'"; exit 1; else exit 0; fi

