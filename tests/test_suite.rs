use draft::test_draft;

test_draft!("tests/suite/tests/draft4/", {
    "optional_bignum_0_0",
    "optional_bignum_2_0",
});
test_draft!("tests/suite/tests/draft6/");
test_draft!("tests/suite/tests/draft7/", {
    "optional_format_idn_hostname_0_11", // https://github.com/Stranger6667/jsonschema-rs/issues/101
    "optional_format_idn_hostname_0_6",  // https://github.com/Stranger6667/jsonschema-rs/issues/101
    "optional_format_idn_hostname_0_7",  // https://github.com/Stranger6667/jsonschema-rs/issues/101
});
