import pytest
from io import StringIO
import sys
from hello import main


@pytest.mark.unit
def test_main_prints_hello():
    captured_output = StringIO()
    sys.stdout = captured_output

    main()

    sys.stdout = sys.__stdout__
    assert captured_output.getvalue() == "Hello from vision!\n"


@pytest.mark.unit
def test_main_returns_none():
    result = main()
    assert result is None
