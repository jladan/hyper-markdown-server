# Markdown Server Usage

```toml
date = 2023-11-06
tags = ["usage", "md-server"]
```

Running with the `-h` for help gives the string:

```
Usage: hyper-markdown-server [OPTIONS] [ADDR]

Arguments:
  [ADDR]  The address (default: '0.0.0.0:7878')

Options:
  -p, --port <PORT>                  Sets the port (default 7878)
  -w, --webroot <WEB_ROOT>           Sets the webserver rooot
  -s, --static-dir <STATIC_DIR>      Sets the location of static files
  -t, --template-dir <TEMPLATE_DIR>  Sets the location of document templates
  -h, --help                         Print help
  -V, --version                      Print version
```

**Note**: The default adress is 0.0.0.0, which opens the server to your entire
local network.

There are 3 required directories, which can also be set with environment
variables:

- `WEB_ROOT` -- the directory containing markdown files to serve
- `STATIC_DIR` -- the location of "static" files, like `.css` and `.js` files
  used in the templates
- `TEMPLATE_DIR` -- the location of templates for rendering the page layout.

## Templates

We use [Tera](https://keats.github.io/tera/) templates to render html. These are
based on Jinja2: the template format for Django.
There are two required templates:
- `directory.html` (directory listings)
- `markdown.html` (a markdown endpoint)

The context provided to the templates contains the variables:
- `dirtree`: a directory listing containing all files in the WEB_ROOT. Fields:
    + `.dirs`: The list of directories with same structure as `dirtree`;
    + `.files`: The list of files of this directory.
- `dir_contents` (`directory.html`): The contents of the current directory.
- `content` (`markdown.html`): The html string generated from the markdown.
