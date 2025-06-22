# CSS Management System

The page editor now includes a comprehensive CSS management system that allows for both element-specific styling and user-defined custom styles.

## How It Works

### 1. Element-Specific CSS

Each builtin element can have its own CSS file defined in the `config.yaml`:

```yaml
elements:
  - name: tag-cloud
    viewer: builtin://tag-cloud/view.html
    css: builtin://tag-cloud/style.css
    editor: builtin://tag-cloud/editor.html
```

The CSS files are located in the `templates/` directory and are automatically loaded and included in the rendered HTML.

### 2. Template Structure

Templates now use the format `class="element-class @@class@@"` where:
- `element-class` is the base CSS class for the element (e.g., `tag-cloud`, `page-header`)
- `@@class@@` is a placeholder that gets replaced with user-defined CSS classes

Example:
```html
<ul class="tag-cloud @@class@@">
    {% for tag in data.tags %}
    <li>{{tag}}</li>
    {% endfor %}
</ul>
```

### 3. User-Defined Styles

In your YAML page definition, you can add custom styles to any element:

```yaml
- type: tag-cloud
  data:
    tags: ["CSS", "Modern Design", "Hover Effects"]
  style:
    margin: 2rem 0
    background: #f0f0f0
    border-radius: 8px
```

### 4. CSS Generation

The system generates unique CSS classes for user-defined styles based on the element's position in the page tree. For example:
- `root_header1` for the first header element
- `root_section2` for the second section element
- `root_tag_cloud3` for the third tag-cloud element

## Builtin Elements and Their CSS

### Tag Cloud (`tag-cloud`)
- **Base class**: `.tag-cloud`
- **Features**: Flexbox layout, gradient background, hover effects on tags
- **File**: `templates/tag-cloud/style.css`

### Header (`header`)
- **Base class**: `.page-header`
- **Features**: Gradient text, underline decoration, hover animation
- **File**: `templates/header/style.css`

### Section (`section`)
- **Base class**: `.content-section`
- **Features**: Card-like design, top border accent, hover effects
- **File**: `templates/section/style.css`

### Text/Paragraph (`text`, `paragraph`)
- **Base class**: `.rich-text`
- **Features**: Typography styling, link styles, code block formatting
- **File**: `templates/text/style.css`

### Image (`image`)
- **Base class**: `.content-image`
- **Features**: Responsive design, hover effects, caption support
- **File**: `templates/image/style.css`

## Example Usage

```yaml
template: plain
title: "My Page"
data:
  - type: header
    data:
      text: "Welcome"
    style:
      text-align: center
      color: #2c3e50
  
  - type: tag-cloud
    data:
      tags: ["Design", "CSS", "Modern"]
    style:
      margin: 2rem 0
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)
```

## Benefits

1. **Separation of Concerns**: Element styling is separated from user customization
2. **Consistency**: All instances of an element type share the same base styling
3. **Flexibility**: Users can override or extend base styles with custom CSS
4. **Maintainability**: Element styles are centralized in dedicated CSS files
5. **Performance**: CSS is generated once and reused across all elements of the same type

## Adding New Elements

To add a new element with CSS support:

1. Create the template file: `templates/my-element/view.html`
2. Create the CSS file: `templates/my-element/style.css`
3. Add the element definition to `config.yaml` with the `css` field
4. Use `class="my-element-class @@class@@"` in your template

The CSS system will automatically load and include your element's styles in the rendered output. 