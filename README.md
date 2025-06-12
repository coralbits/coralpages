# Page Editor

This is an API first page editor and renderer.

The main functionality is a full featured page editor, using an embedable SPA, and it generates
HTML on an API endpoint.

The generated HTML can be from the builtin elements (section, block, image, button, rich text,
video...), or from another external microservice via REST API.

## Builtint Elements

- Section
- Block
- Image
- Button
- Rich Text
- Video

## External Elements

To add external elements edit the config.yaml and add the endpoints for the external elements. It
must have thse format:

```yaml
external:
  - name: Login Button
    renderer: http://localhost:8080/api/login-button
    editor: http://localhost:8080/api/login-button-editor
```

Actually all internal elements are defined in the same way in the config file, and can be changed
if necessary.

The render function will receive a JSON object with the definition as given by the editor, as well
as page context, and must return the HTML content.

The page collector will use cache as possible to ensure its not rendered more than necesary, using
the standard "expires" header.

The editor must be an HTML form, using internally whatever is desired. Initial values are passed
to the constructor as well.
