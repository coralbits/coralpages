# Page Viewer

## Overview

This is an API first page renderer. There is no editor, nor editing API yet.

It has several endpoints that allow to assemble pages.

- /api/v1/view/{page_name}

It has a nested approach to page creation. Basic mode is just using
builtin blocks, passing some data, and rendering the result. The
blocks can be nested themselves. The page is rendered as a single HTML
page, with the blocks rendered as if they were in the page.

On a more advanced level, the blocks can be requested to be rendered by
remote endpoints, and the result will be rendered as if it were a builtin
block.

This is structured in stores, which can return HTML with jinja templating,
or raw HTML. Each store has different configuration, and can be mixed.

Stores have several functions:

- Pages
- Blocks
- Templates

## Builtin Blocks Templates

- Section
- Block
- Image
- Button
- Rich Text
- Markdown
- Video
- Header
- Tag Cloud
- Menu
- Paragraph

## External Blocks Templates

To add external blocks edit the config.yaml and add the endpoints for the external blocks. It
must have these format:

```yaml
stores:
  - name: myremote
    type: http
    base_url: http://localhost:8080/api/v1/element/
    tags:
      - pages
      - blocks
      - templates
```

## Config structure and data flow

In the config there are several stores configured, each store is a loader that can get data in a way, from filesystem, database, or HTTP.

When viewing a page all stores in order are queried for the given page, first to answer is used.

The page has blocks, for the block we follow the same idea, but first we
check into the config one, and then maybe on the store.

Each store has its own configuration and may not have blocks pages or
whatever. We use tags to filter out when not needed.

Once we have the definitions, we proceed to render each block, in
parallel, and finally assemble the final page.

The page may have a template, which do the same process, only for
template stores, and sets the children.

The sequence diagram is the following:

```mermaid
sequenceDiagram
  User ->>+ Renderer: Request page
  Renderer <<->>+ Loader: Load page
  Loader <<->>- Store: Load page from store
  Renderer <<->>+ Loader: Load block
  Loader <<->>- Store: Load block from store
  Renderer <<->>+ Loader: Load block
  Loader <<->>- Store: Load block from store
  Renderer <<->>+ Loader: Load template
  Loader <<->>- Store: Load template from store
  Renderer ->>- User: Render response
```

The renderer may need to change the response in some ways, for example
jinja templating, or nothing.

The loader can actually use the block data to render it itself and do not
require any more templating.

## Terminology

### Store

Its a repository for block templates, pages and other resources.

### BlockTemplate

Its the blueprint of a block. They contains some HTML, CSS, and may add a context.

### Block

Its a definition of a block of a page, using a BlockTemplate, giving some specific properties.
For example a title will set the exact title, and maybe some CSS properties.

### RenderedBlock

Its the rendered result of a block.

### Page

Its the definition of a specific page. It contains many Blocks that will use BlockTemplates
to render themselves.

They can be recursive via templates, so a page decides the important blocks at that level,
and the template wraps it all inside a template. For example you can create
a blog page using the blog template, and a product page using the product template.

Templates can use templates themselves, and finally are just rendered to an html using
a template. this allows even to create a full html based template where you put your blocks.

### RenderedPage

Its the rendered result of a page template.
