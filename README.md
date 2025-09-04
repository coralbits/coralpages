# Page Viewer

## Overview

This is an API first page renderer. There is no editor. For that check out the
[page editor](https://github.com/coralbits/pe).

The main idea is to be used as a microservice focused only in ease the edition (via the editor),
and very fast specialized rendering of the HTML content.

Currently I'm reimplementing the whole system in rust for performance reasons. The initial
prototype in Python proved very useful, in rust its extremely usefull as the speed increase is
incredible (10x in my tests, normally sub ms for a full page render).

## Features

- [ ] Flexible store configuration support. Can have sveral stores, each with a diferent backend.
- [ ] Easy to add new widgets to create a custom page personality.
- [ ] Easy to add new stores to create a custom page backend.
- [ ] HTTP backend, so it can add data to the context, or directly render elements.

## TODO

- [ ] Permissions per store. So some JWT tags can be used to allow view or edit.
- [ ] Draft support & History. Now only last edited page is stored in the stores.
- [ ] Improve HTTP support for the rust implemetnation.

## License

Page Viewer is licensed under the AGPLv3 license.

This basically means that any of your users, even on netowrk enviroments, have the
right to get the source code of the software and any modification.

If for any reason this is not acceptable to you or your company, it's possible to
purchase a commercial license from Coralbits, with a fixed closed price of 100€ per
release. This basically means that you can use the software with the only limitation
is that relicensing, redistribution nor resale are allowed.

If you need a commercial license, please contact us at info@coralbits.com.

## Terminology

- Store: A repository for widgets, pages and other resources.
- Widget: A blueprint of an element. They contains some HTML, CSS, and may add a context.
- Element: A definition of an element of a page, using a BlockTemplate, giving some specific properties.
- RenderedElement: The rendered result of an element.
- Page: A definition of a specific page. It contains many Elements that will use Widgets
  to render themselves.
- RenderedPage: The rendered result of a page template.

## Basic endpoints

- `POST /api/v1/render/` -- Render a page. Send the page json, and receives a json of the html
  parts. Can ask for a given format or part like only the html, CSS or JS, of all the parts in
  JSON. My preferred way is the JSON format with the parts, so some very simple templating in the frontend can place the parts where needed.
- `GET/PUT/POST /api/v1/page/:store/:path` -- Get/create/modify a page. If it doesn't exist, it will
  be created with the default page template.
- `GET /api/v1/render/:store/:path` -- Get a rendered page from the page store.
- `GET /docs` -- Full OpenAPI documentation of the API.

There are other paths that are necesary for the page editor, like list of available widgets or
stores.

## Page structure

There are several example pages at `builtin/pages`. These are the default pages which contains
more detailed documentation.

But a very basic page would be, in JSON:

```json
{
  "title": "Example page",
  "children": [
    {
      "type": "builtin/markdown",
      "content": "This is an example page."
    }
  ]
}
```
