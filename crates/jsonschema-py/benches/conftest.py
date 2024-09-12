def pytest_configure(config):
    config.addinivalue_line("markers", "data(schema, instance): add data for benchmarking")
