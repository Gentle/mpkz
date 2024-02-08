import mpkz

def test_dump_load_buffer():
    input = [1, True, None, { "hello": "world" }, 1.5]
    binary = mpkz.dumpb(input)
    output = mpkz.loadb(binary)
    assert input == output

def test_dump_load_file(tmp_path):
    input = [1, True, None, { "hello": "world" }, 1.5]
    with open(tmp_path / "x", "wb") as f:
        mpkz.dump(input, f)
    with open(tmp_path / "x", "rb") as f:
        output = mpkz.load(f)
    assert input == output

def test_open_buffer():
    input = [1, True, None, { "hello": "world" }, 1.5]
    binary = mpkz.dumpb(input)
    output = list(mpkz.openb(binary))
    assert input == output

def test_create_file(tmp_path):
    filename = tmp_path / "archive"
    with mpkz.create(filename) as writer:
        writer.append(1)
        writer.append(2)
        writer.extend([3, 4, 5])
        writer.extend((6, 7))
        writer.extend([i for i in range(8, 10)])
    output = list(mpkz.open(filename))
    assert output == [1, 2, 3, 4, 5, 6, 7, 8, 9]
