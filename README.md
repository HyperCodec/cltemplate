# CLTemplate
[<img alt="github" src="https://img.shields.io/github/last-commit/hypercodec/cltemplate" height="20">](https://github.com/hypercodec/cltemplate)
[<img alt="crates.io" src="https://img.shields.io/crates/d/cltemplate" height="20">](https://crates.io/crates/cltemplate)

Small commandline tool to create and use templates quickly

### Installation
Simply run `cargo install cltemplate` to install the tool.

### Using Templates
To use a template, do the following:
1. CD into the folder of the template (your terminal should be running at `path/to/template`)
2. Run `template path/to/output` and check the directory you provided

Optionally, you can use the `--template-path` param (`-t` for short) instead of CDing to the the template folder

### Writing Templates
To write a template that others can use with this tool, you must:
1. Create a folder for the template
2. In the root directory of the folder, create a `template.txt` file. In this file, there should be the name of each replaceable item, separated by newlines (ex: 
    ```
    foo
    bar
    buz
    qux
    quux
    corge
    grault
    garply
    waldo
    fred
    plugh
    xyzzy
    thud
    ```
    )

3. Fill in the rest of the folder with the files provided in the template. Anything defined in `template.txt` that has `{% %}` around it (ex: `{% foo %}`) will be replaced by whatever the user inputs. Anything surrounded by `{% %}` that is not included in `template.txt` will remain unchanged.
4. Share your template and usage instructions via your preferred method.

### License
This tool is licensed under the `MIT` license.

### Roadmap
- [x] ~~Initial prototype~~
- [x] ~~Complete rework~~
- [x] ~~Asynchronous file IO~~
- [ ] [Indicatif task indicators](https://github.com/HyperCodec/cltemplate/issues/14)
- [ ] Optionally providing template args in cli
- [ ] Dynamic filepaths