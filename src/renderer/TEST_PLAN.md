# Renderer Test Plan

This document outlines the comprehensive test plan for the Coralpages renderer module.

## T001 - Very basic page render
* [x] Create a test page with simple elements
* [x] Render it using the renderer
* [x] Ensure the contents match expected output
* **Status**: ✅ Implemented as `test_basic_page_render`

## T002 - Render page with section with children
* [x] Create a test page with nested elements (section with children)  
* [x] Verify that rendering processes children recursively
* [x] Ensure the children are rendered in correct order and structure
* **Status**: ✅ Implemented as `test_page_with_nested_children`

## T003 - Render with styles
* [x] Create a test page with custom inline styles for elements
* [x] Verify rendering does not fail
* [x] Ensure the defined styles are present in the rendered CSS output
* **Status**: ✅ Implemented as `test_page_with_custom_styles`

## T004 - Render with custom classes
* [x] Create mock CSS class definitions in test store
* [x] Create a test page using custom CSS classes
* [x] Verify custom classes are loaded and applied to rendered page
* [x] Ensure CSS class definitions are included in output
* **Status**: ✅ Implemented as `test_page_with_custom_classes`

## T005 - Error handling - Missing widget
* [x] Create a page referencing a non-existent widget
* [x] Verify appropriate error is returned
* [x] Test both debug and non-debug modes
* **Status**: ✅ Implemented as `test_missing_widget_error`

## T006 - Error handling - Template rendering errors
* [x] Create a widget with invalid template syntax
* [x] Verify error handling during template rendering
* [x] Test both debug and non-debug modes (debug should show error in HTML)
* **Status**: ✅ Implemented as `test_template_rendering_errors`

## T007 - Meta data handling
* [x] Create a page with meta definitions
* [x] Verify meta data is properly transferred to rendered page
* [x] Test multiple meta tags
* **Status**: ✅ Implemented as `test_meta_data_handling`

## T008 - Data context templating
* [x] Create elements with templated data values (using {{ }} syntax)
* [x] Verify template variables are resolved correctly
* [x] Test nested context resolution
* **Status**: ✅ Implemented as `test_data_context_templating`

## T009 - Static context widgets
* [ ] Test rendering of `static_context` widget type
* [ ] Test rendering of `url_context` widget type
* [ ] Verify context is properly resolved for these special widgets
* **Status**: ⏳ Not yet implemented - requires static/url context widget setup

## T010 - Full HTML page generation
* [x] Test `render_full_html_page()` method
* [x] Verify complete HTML structure (DOCTYPE, head, body)
* [x] Ensure CSS is properly embedded in style tags
* [x] Test viewport meta tag inclusion
* **Status**: ✅ Implemented as `test_full_html_page_generation`

## T011 - CSS variable generation
* [x] Test CSS variable generation from multiple sources
* [x] Verify proper CSS formatting and escaping
* [x] Test CSS sorting functionality
* **Status**: ✅ Implemented as `test_css_variable_generation`

## T012 - Complex nested rendering
* [x] Create deeply nested element structures
* [x] Verify all levels render correctly
* [x] Test performance with complex structures
* **Status**: ✅ Implemented as `test_complex_nested_rendering`

## T013 - Widget CSS integration
* [x] Test that widget CSS is properly added to rendered page
* [x] Verify CSS variable naming conventions
* [x] Test CSS deduplication
* **Status**: ✅ Implemented as `test_widget_css_integration`

## T014 - Element ID and styling
* [x] Test elements with IDs get proper CSS ID selectors
* [x] Verify inline styles are converted to CSS rules
* [x] Test CSS property formatting
* **Status**: ✅ Covered in `test_page_with_custom_styles` and `test_css_variable_generation`

## T015 - Debug mode behavior
* [x] Test error display in debug mode (red error boxes)
* [x] Test error suppression in non-debug mode
* [x] Verify error collection in rendered page
* **Status**: ✅ Implemented as `test_debug_mode_behavior`

## T016 - Context inheritance
* [x] Test context passing between parent and child elements
* [x] Verify context merging with element-specific data
* [x] Test context variable precedence
* **Status**: ✅ Covered in `test_data_context_templating` and nested rendering tests

## T017 - Performance and timing
* [x] Test elapsed time tracking
* [x] Verify performance with large page structures
* [ ] Test memory usage patterns
* **Status**: ✅ Implemented as `test_performance_timing`

## T018 - Response codes and headers
* [x] Test default response code (200)
* [x] Test custom headers functionality
* [x] Verify header persistence through rendering
* **Status**: ✅ Implemented as `test_response_codes_and_headers`

## Helper Functions Needed

### YAML Page Definition Parser
* `parse_page_from_yaml(yaml: &str) -> Page` - Convert YAML to Page object ✅
* `parse_element_from_yaml_value(yaml: &serde_yaml::Value) -> Element` - Convert YAML to Element ✅
* `create_test_widget(name: &str, html: &str, css: &str) -> Widget` - Quick widget creation ✅

### Test Store Utilities
* Enhanced TestStore with configurable widgets and CSS classes ✅
* Mock error conditions for testing error handling ✅
* Configurable delays for performance testing ⏳

### Assertion Helpers
* `assert_html_structure(html: &str, expected_structure: &str)` ✅
* `assert_meta_tags(rendered: &RenderedPage, expected: &[MetaDefinition])` ✅

## Test Data Organization

Tests should use consistent, well-documented test data:
* Simple pages for basic functionality ✅
* Complex nested structures for stress testing ✅
* Invalid/error cases for robustness testing ✅
* Real-world page examples for integration testing ✅ (YAML-based test data)

## Summary

**Implemented**: 15/18 test cases (83% coverage)
**Remaining**: 3 test cases
- T009: Static context widgets (requires additional setup)
- T017: Memory usage patterns (partially implemented)
- Enhanced performance testing features