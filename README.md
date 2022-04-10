# gm2h
![Logo](./images/logo.png)

Generate markdown to HTML.

This program that automatically converts markdown files to HTML files when they are saved.

## Demo
![Demo](./images/demo.gif)

## Usage

```sh
# Create index.md
touch index.md

# Run gm2h
gm2h
```

Let's edit the markdown file.  
An generated HTML file is generated in the current directory.

directory can also be specified for use. See `--help` for details.

```sh
gm2h --help
```