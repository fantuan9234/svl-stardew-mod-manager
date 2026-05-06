import pytest

def pytest_addoption(parser):
    parser.addoption(
        "--svl-url",
        action="store",
        default="http://localhost:1420",
        help="SVL 应用 URL"
    )

@pytest.fixture(scope="function")
def svl_url(request):
    return request.config.getoption("--svl-url")
