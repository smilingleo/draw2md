# DrawIO Diagram to Markdown

This is a tool to parse a DrawIO exported XML file and generate the markdown doc. The gist is that you only need to maintain the diagrams.

## How to use

Draw any diagram on [DrawIO](http://draw.io), add your note to:

1. On the diagram page (not to select any object), add `note` attribute.
2. Select any element, add `tooltip` attribute.
3. Export the diagram as XML file.
4. Export the diagram as PNG images. (Not necessary, it should be able to generate SVG/PNG directly adcording to the XML, save it for later implementation).
5. Run this tool.
    
    ```
    draw2md --assets <folder> --name <DiagramName>
    ```

## Features

* Image as data url, so you only see one markdown file.
* Organize objects by tab.

Enjoy!