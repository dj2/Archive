```
def a
  x = x + 1
  x
end
```

``` glsl
#version 460

layout (location = 0) out vec4 frag_color;

void main() {
  frag_color = vec4(1.0, 0.2, 0.8, 1.0);
}
```

    Just a paragraph, no indented code blocks.
    With two lines.


    ``` rust
    fn test() -> bool {
      1 == 2
    }
    ```

````
A shorter marker.
```
``````

~~~
Different marker
```
~~~
