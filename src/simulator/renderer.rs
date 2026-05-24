use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
};
use js_sys::{Float32Array, Uint8Array};

pub struct Renderer {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    texture: WebGlTexture,
    buffer: WebGlBuffer,
    width: i32,
    height: i32,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement, width: usize, height: usize) -> Result<Self, JsValue> {
        let gl = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("WebGL2 context not supported"))?
            .dyn_into::<WebGl2RenderingContext>()?;

        // 1. Create Shaders
        let vertex_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            r#"#version 300 es
            in vec2 position;
            out vec2 v_tex_coords;
            void main() {
                v_tex_coords = position * 0.5 + 0.5;
                v_tex_coords.y = 1.0 - v_tex_coords.y; // Match screen space where y=0 is top
                gl_Position = vec4(position, 0.0, 1.0);
            }
            "#,
        )?;

        let fragment_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es
            precision highp float;
            in vec2 v_tex_coords;
            out vec4 out_color;
            uniform sampler2D u_texture;
            void main() {
                out_color = texture(u_texture, v_tex_coords);
            }
            "#,
        )?;

        // 2. Link Program
        let program = gl.create_program().ok_or("Failed to create WebGL program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        if !gl
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let log = gl.get_program_info_log(&program).unwrap_or_default();
            return Err(JsValue::from_str(&format!("Shader link failed: {}", log)));
        }

        gl.use_program(Some(&program));

        // 3. Set up Quad Vertices (2 Triangles forming a full canvas quad)
        let vertices: [f32; 8] = [
            -1.0, -1.0,
             1.0, -1.0,
            -1.0,  1.0,
             1.0,  1.0,
        ];

        let buffer = gl.create_buffer().ok_or("Failed to create WebGL buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Upload vertices using view for speed
        let view = unsafe { Float32Array::view(&vertices) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &view,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // Bind position attribute
        let pos_attrib = gl.get_attrib_location(&program, "position");
        if pos_attrib >= 0 {
            let pos_attrib = pos_attrib as u32;
            gl.enable_vertex_attrib_array(pos_attrib);
            gl.vertex_attrib_pointer_with_i32(
                pos_attrib,
                2,
                WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        }

        // 4. Set up Texture
        let texture = gl.create_texture().ok_or("Failed to create WebGL texture")?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        // Use NEAREST filtering for crisp pixel art looks (no blur)
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );

        // Allocate empty buffer space initially
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width as i32,
            height as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            None,
        )?;

        Ok(Renderer {
            gl,
            program,
            texture,
            buffer,
            width: width as i32,
            height: height as i32,
        })
    }

    pub fn draw(&self, pixel_buffer: &[u8]) -> Result<(), JsValue> {
        let gl = &self.gl;

        // Use our program
        gl.use_program(Some(&self.program));

        // Bind our vertex array buffer
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));

        // Update texture data with the cell colors
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        
        let view = unsafe { Uint8Array::view(pixel_buffer) };
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            self.width,
            self.height,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&view),
        )?;

        // Update viewport based on actual canvas size
        let canvas = gl.canvas().unwrap().dyn_into::<HtmlCanvasElement>()?;
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        // Clear and Draw the quad
        gl.clear_color(0.04, 0.05, 0.06, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        Ok(())
    }
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if !gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        return Err(JsValue::from_str(&format!(
            "Shader compile failed: {}",
            log
        )));
    }

    Ok(shader)
}
