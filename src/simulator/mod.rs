pub mod element;
pub mod grid;
pub mod renderer;

use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

use crate::simulator::element::ElementType;
use crate::simulator::grid::Grid;
use crate::simulator::renderer::Renderer;

pub struct Simulator {
    pub grid: Grid,
    renderer: Renderer,
    color_buffer: Vec<u8>,
}

impl Simulator {
    pub fn new(canvas: &HtmlCanvasElement, width: usize, height: usize) -> Result<Self, JsValue> {
        let grid = Grid::new(width, height);
        let renderer = Renderer::new(canvas, width, height)?;
        let color_buffer = vec![0; width * height * 4];

        Ok(Simulator {
            grid,
            renderer,
            color_buffer,
        })
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        self.grid.get_color_buffer(&mut self.color_buffer);
        self.renderer.draw(&self.color_buffer)?;
        Ok(())
    }

    pub fn active_particle_count(&self) -> usize {
        self.grid
            .cells
            .iter()
            .filter(|c| c.element != ElementType::Air)
            .count()
    }
}
