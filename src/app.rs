use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, MouseEvent, TouchEvent};
use std::cell::RefCell;
use std::rc::Rc;

use crate::simulator::element::ElementType;
use crate::simulator::Simulator;

const GRID_WIDTH: usize = 240;
const GRID_HEIGHT: usize = 180;

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = create_node_ref::<leptos::html::Canvas>();

    // Reactivity Signals
    let (selected_element, set_selected_element) = create_signal(ElementType::Sand);
    let (brush_size, set_brush_size) = create_signal(5);
    let (paused, set_paused) = create_signal(false);
    let (speed, set_speed) = create_signal(1); // simulation steps per frame
    let (fps, set_fps) = create_signal(0);
    let (particles, set_particles) = create_signal(0);
    let (cursor_pos, set_cursor_pos) = create_signal(None::<(f64, f64)>);

    // Simulator references shared with tick closures and event handlers
    let simulator = Rc::new(RefCell::new(None::<Simulator>));

    // Mouse drawing state
    let drawing = Rc::new(RefCell::new(false));
    let drawing_clone = drawing.clone();

    // Helper to map screen mouse client coords to grid coords (240x180)
    let get_coords = move |canvas: &HtmlCanvasElement, client_x: i32, client_y: i32| -> Option<(usize, usize)> {
        let rect = canvas.get_bounding_client_rect();
        let x_rel = client_x as f64 - rect.left();
        let y_rel = client_y as f64 - rect.top();

        let grid_x = (x_rel / rect.width() * GRID_WIDTH as f64) as i32;
        let grid_y = (y_rel / rect.height() * GRID_HEIGHT as f64) as i32;

        if grid_x >= 0 && grid_x < GRID_WIDTH as i32 && grid_y >= 0 && grid_y < GRID_HEIGHT as i32 {
            Some((grid_x as usize, grid_y as usize))
        } else {
            None
        }
    };

    // Helper to map first touch coords
    let get_touch_coords = move |canvas: &HtmlCanvasElement, event: &TouchEvent| -> Option<(usize, usize)> {
        let touches = event.touches();
        if touches.length() > 0 {
            let touch = touches.item(0)?;
            get_coords(canvas, touch.client_x(), touch.client_y())
        } else {
            None
        }
    };

    // Initialize simulation on canvas load
    canvas_ref.on_load({
        let simulator = simulator.clone();
        move |canvas_el: leptos::HtmlElement<leptos::html::Canvas>| {
            use std::ops::Deref;
            let canvas: &HtmlCanvasElement = canvas_el.deref();

            // Set internal buffer dimensions (scaled by CSS)
            canvas.set_width(GRID_WIDTH as u32);
            canvas.set_height(GRID_HEIGHT as u32);

            let mut sim = Simulator::new(canvas, GRID_WIDTH, GRID_HEIGHT).unwrap();
            
            // Seed initial map with terrain
            sim.grid.generate_terrain();
            
            // Set state
            *simulator.borrow_mut() = Some(sim);

            // Set up requestAnimationFrame loop
            let anim_frame = Rc::new(RefCell::new(None));
            let anim_frame_clone = anim_frame.clone();

            let tick_fn = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
            let tick_fn_clone = tick_fn.clone();

            let last_time = web_sys::window().unwrap().performance().unwrap().now();
            let mut frame_count = 0;
            let mut fps_timer = last_time;

            let simulator_loop = simulator.clone();

            let tick_fn_clone_inner = tick_fn_clone.clone();
            *tick_fn_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                let now = web_sys::window().unwrap().performance().unwrap().now();
                frame_count += 1;

                if now - fps_timer >= 1000.0 {
                    set_fps.set(frame_count);
                    frame_count = 0;
                    fps_timer = now;

                    // Update particle count at a lower interval
                    if let Some(ref sim) = *simulator_loop.borrow() {
                        set_particles.set(sim.active_particle_count());
                    }
                }

                // Tick simulation
                if let Some(ref mut sim) = *simulator_loop.borrow_mut() {
                    if !paused.get() {
                        for _ in 0..speed.get() {
                            sim.grid.tick();
                        }
                    }
                    let _ = sim.draw();
                }

                // Request next frame
                let window = web_sys::window().unwrap();
                *anim_frame_clone.borrow_mut() = Some(
                    window
                        .request_animation_frame(
                            tick_fn_clone_inner
                                .borrow()
                                .as_ref()
                                .unwrap()
                                .as_ref()
                                .unchecked_ref(),
                        )
                        .unwrap(),
                );
            }) as Box<dyn FnMut()>));

            // Start loop
            let window = web_sys::window().unwrap();
            *anim_frame.borrow_mut() = Some(
                window
                    .request_animation_frame(
                        tick_fn
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .unwrap(),
            );
        }
    });

    // Event Handlers for Canvas Drawing
    let on_mousedown = {
        let simulator = simulator.clone();
        let drawing = drawing.clone();
        let canvas_ref = canvas_ref.clone();
        move |e: MouseEvent| {
            *drawing.borrow_mut() = true;
            if let Some(canvas) = canvas_ref.get() {
                use std::ops::Deref;
                let raw_canvas: &HtmlCanvasElement = canvas.deref();
                if let Some((gx, gy)) = get_coords(raw_canvas, e.client_x(), e.client_y()) {
                    if let Some(ref mut sim) = *simulator.borrow_mut() {
                        sim.grid.draw_circle(gx, gy, brush_size.get() as usize, selected_element.get());
                        let _ = sim.draw();
                    }
                }
            }
        }
    };

    let on_mousemove = {
        let simulator = simulator.clone();
        let drawing = drawing.clone();
        let canvas_ref = canvas_ref.clone();
        move |e: MouseEvent| {
            if let Some(canvas) = canvas_ref.get() {
                use std::ops::Deref;
                let raw_canvas: &HtmlCanvasElement = canvas.deref();
                
                // Update cursor overlay position
                let rect = raw_canvas.get_bounding_client_rect();
                let px = (e.client_x() as f64 - rect.left()) / rect.width() * 100.0;
                let py = (e.client_y() as f64 - rect.top()) / rect.height() * 100.0;
                set_cursor_pos.set(Some((px, py)));

                if *drawing.borrow() {
                    if let Some((gx, gy)) = get_coords(raw_canvas, e.client_x(), e.client_y()) {
                        if let Some(ref mut sim) = *simulator.borrow_mut() {
                            sim.grid.draw_circle(gx, gy, brush_size.get() as usize, selected_element.get());
                            let _ = sim.draw();
                        }
                    }
                }
            }
        }
    };

    let on_mouseup = {
        let drawing = drawing.clone();
        move |_| {
            *drawing.borrow_mut() = false;
        }
    };

    let on_mouseleave = {
        let drawing = drawing.clone();
        move |_| {
            *drawing.borrow_mut() = false;
            set_cursor_pos.set(None);
        }
    };

    // Touch support (Mobile)
    let on_touchstart = {
        let simulator = simulator.clone();
        let drawing = drawing_clone.clone();
        let canvas_ref = canvas_ref.clone();
        move |e: TouchEvent| {
            e.prevent_default();
            *drawing.borrow_mut() = true;
            if let Some(canvas) = canvas_ref.get() {
                use std::ops::Deref;
                let raw_canvas: &HtmlCanvasElement = canvas.deref();
                if let Some((gx, gy)) = get_touch_coords(raw_canvas, &e) {
                    if let Some(ref mut sim) = *simulator.borrow_mut() {
                        sim.grid.draw_circle(gx, gy, brush_size.get() as usize, selected_element.get());
                        let _ = sim.draw();
                    }
                }
            }
        }
    };

    let on_touchmove = {
        let simulator = simulator.clone();
        let drawing = drawing_clone.clone();
        let canvas_ref = canvas_ref.clone();
        move |e: TouchEvent| {
            e.prevent_default();
            if let Some(canvas) = canvas_ref.get() {
                use std::ops::Deref;
                let raw_canvas: &HtmlCanvasElement = canvas.deref();
                
                if let Some(touches) = e.touches().item(0) {
                    let rect = raw_canvas.get_bounding_client_rect();
                    let px = (touches.client_x() as f64 - rect.left()) / rect.width() * 100.0;
                    let py = (touches.client_y() as f64 - rect.top()) / rect.height() * 100.0;
                    set_cursor_pos.set(Some((px, py)));
                }

                if *drawing.borrow() {
                    if let Some((gx, gy)) = get_touch_coords(raw_canvas, &e) {
                        if let Some(ref mut sim) = *simulator.borrow_mut() {
                            sim.grid.draw_circle(gx, gy, brush_size.get() as usize, selected_element.get());
                            let _ = sim.draw();
                        }
                    }
                }
            }
        }
    };

    let on_touchend = {
        let drawing = drawing_clone.clone();
        move |e: TouchEvent| {
            e.prevent_default();
            *drawing.borrow_mut() = false;
            set_cursor_pos.set(None);
        }
    };

    // Actions
    let handle_clear = {
        let simulator = simulator.clone();
        move |_| {
            if let Some(ref mut sim) = *simulator.borrow_mut() {
                sim.grid.clear();
                let _ = sim.draw();
            }
        }
    };

    let handle_terrain = {
        let simulator = simulator.clone();
        move |_| {
            if let Some(ref mut sim) = *simulator.borrow_mut() {
                sim.grid.generate_terrain();
                let _ = sim.draw();
            }
        }
    };

    let handle_step = {
        let simulator = simulator.clone();
        move |_| {
            if let Some(ref mut sim) = *simulator.borrow_mut() {
                sim.grid.tick();
                let _ = sim.draw();
            }
        }
    };

    // Element categorizations
    let solids = vec![
        ElementType::Sand,
        ElementType::Gravel,
        ElementType::Gunpowder,
        ElementType::Gold,
        ElementType::Stone,
        ElementType::Wood,
        ElementType::Coal,
        ElementType::Ice,
    ];

    let liquids = vec![
        ElementType::Water,
        ElementType::Oil,
        ElementType::Acid,
        ElementType::Lava,
    ];

    let gases = vec![
        ElementType::Fire,
        ElementType::Smoke,
        ElementType::Steam,
    ];

    view! {
        <style>
            r#"
            .dashboard {
                display: flex;
                flex-direction: column;
                gap: 20px;
                width: 95vw;
                max-width: 1400px;
                height: 92vh;
                max-height: 900px;
                background: var(--bg-surface);
                backdrop-filter: blur(16px);
                border: 1px solid var(--border-color);
                box-shadow: 0 16px 40px rgba(0, 0, 0, 0.4);
                border-radius: 20px;
                padding: 24px;
                overflow: hidden;
            }

            .header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                border-bottom: 1px solid var(--border-color);
                padding-bottom: 16px;
            }

            .title-area h1 {
                font-family: var(--font-display);
                font-size: 28px;
                font-weight: 800;
                background: linear-gradient(135deg, #ffffff, #888888);
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
                text-shadow: 0 2px 10px rgba(255, 255, 255, 0.05);
                letter-spacing: 1px;
            }

            .title-area p {
                font-size: 11px;
                color: var(--primary-neon);
                text-transform: uppercase;
                letter-spacing: 2px;
                margin-top: 2px;
                font-weight: 600;
                text-shadow: 0 0 10px var(--primary-neon-glow);
            }

            .main-content {
                display: grid;
                grid-template-columns: 280px 1fr 280px;
                gap: 20px;
                flex: 1;
                overflow: hidden;
            }

            .sidebar {
                background: rgba(8, 10, 15, 0.5);
                border: 1px solid var(--border-color);
                border-radius: 12px;
                padding: 16px;
                display: flex;
                flex-direction: column;
                gap: 20px;
                overflow-y: auto;
            }

            .sidebar h2 {
                font-family: var(--font-display);
                font-size: 14px;
                font-weight: 700;
                letter-spacing: 1px;
                color: #e2e8f0;
                border-left: 3px solid var(--primary-neon);
                padding-left: 8px;
                margin-bottom: 8px;
                text-transform: uppercase;
            }

            .elem-group {
                display: flex;
                flex-direction: column;
                gap: 8px;
            }

            .elem-grid {
                display: grid;
                grid-template-columns: repeat(2, 1fr);
                gap: 8px;
            }

            .elem-btn {
                background: rgba(255, 255, 255, 0.02);
                border: 1px solid var(--border-color);
                border-radius: 8px;
                padding: 10px 8px;
                display: flex;
                align-items: center;
                gap: 8px;
                color: #94a3b8;
                font-size: 12px;
                font-weight: 500;
                cursor: pointer;
                transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
            }

            .elem-btn:hover {
                background: var(--bg-surface-hover);
                color: #fff;
                border-color: rgba(255, 255, 255, 0.2);
                transform: translateY(-1px);
            }

            .elem-btn.active {
                background: rgba(0, 240, 255, 0.08);
                border-color: var(--primary-neon);
                color: #fff;
                box-shadow: 0 0 12px var(--primary-neon-glow);
            }

            .color-dot {
                width: 12px;
                height: 12px;
                border-radius: 3px;
                border: 1px solid rgba(0,0,0,0.3);
                box-shadow: 0 1px 3px rgba(0,0,0,0.4);
                flex-shrink: 0;
            }

            .center-panel {
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: 16px;
                overflow: hidden;
            }

            .canvas-wrapper {
                position: relative;
                width: 100%;
                max-width: 640px;
                aspect-ratio: 240 / 180;
                background: #000;
                border: 2px solid var(--border-color);
                border-radius: 12px;
                overflow: hidden;
                box-shadow: inset 0 0 20px rgba(0,0,0,0.8), 0 0 30px rgba(0,0,0,0.5);
                cursor: crosshair;
                transition: border-color 0.3s;
            }

            .canvas-wrapper:hover {
                border-color: rgba(255, 255, 255, 0.15);
            }

            .sim-canvas {
                width: 100%;
                height: 100%;
                display: block;
                image-rendering: pixelated;
                image-rendering: crisp-edges;
            }

            .brush-preview {
                position: absolute;
                border: 1.5px dashed var(--primary-neon);
                border-radius: 50%;
                pointer-events: none;
                box-shadow: 0 0 6px var(--primary-neon-glow);
                background: rgba(0, 240, 255, 0.05);
            }

            .control-panel {
                width: 100%;
                max-width: 640px;
                background: rgba(8, 10, 15, 0.5);
                border: 1px solid var(--border-color);
                border-radius: 12px;
                padding: 16px;
                display: flex;
                flex-direction: column;
                gap: 12px;
            }

            .control-row {
                display: flex;
                justify-content: space-between;
                align-items: center;
                gap: 12px;
                flex-wrap: wrap;
            }

            .btn-group {
                display: flex;
                gap: 8px;
            }

            .control-btn {
                background: rgba(255, 255, 255, 0.04);
                border: 1px solid var(--border-color);
                border-radius: 8px;
                padding: 8px 14px;
                color: #e2e8f0;
                font-size: 13px;
                font-weight: 500;
                cursor: pointer;
                transition: all 0.2s;
                display: flex;
                align-items: center;
                gap: 6px;
            }

            .control-btn:hover {
                background: rgba(255, 255, 255, 0.08);
                color: #fff;
                border-color: rgba(255, 255, 255, 0.2);
            }

            .control-btn.active {
                background: var(--danger-neon);
                border-color: var(--danger-neon);
                color: #fff;
                box-shadow: 0 0 10px rgba(255, 0, 85, 0.3);
            }

            .control-btn.primary {
                background: var(--primary-neon);
                border-color: var(--primary-neon);
                color: #050608;
                font-weight: 600;
                box-shadow: 0 0 12px var(--primary-neon-glow);
            }

            .control-btn.primary:hover {
                background: #33f3ff;
                box-shadow: 0 0 18px rgba(0, 240, 255, 0.4);
            }

            .slider-container {
                display: flex;
                align-items: center;
                gap: 10px;
                color: #94a3b8;
                font-size: 12px;
            }

            .slider-container input[type=range] {
                -webkit-appearance: none;
                width: 100px;
                background: rgba(255,255,255,0.08);
                height: 4px;
                border-radius: 2px;
                outline: none;
            }

            .slider-container input[type=range]::-webkit-slider-thumb {
                -webkit-appearance: none;
                width: 12px;
                height: 12px;
                border-radius: 50%;
                background: var(--primary-neon);
                cursor: pointer;
                box-shadow: 0 0 4px var(--primary-neon-glow);
            }

            .info-panel {
                display: flex;
                flex-direction: column;
                gap: 16px;
            }

            .card {
                background: rgba(8, 10, 15, 0.4);
                border: 1px solid var(--border-color);
                border-radius: 12px;
                padding: 16px;
            }

            .card-title {
                font-family: var(--font-display);
                font-size: 13px;
                font-weight: 700;
                text-transform: uppercase;
                color: #e2e8f0;
                letter-spacing: 0.5px;
                margin-bottom: 12px;
                border-bottom: 1px solid rgba(255,255,255,0.05);
                padding-bottom: 6px;
            }

            .stat-grid {
                display: flex;
                flex-direction: column;
                gap: 10px;
                font-family: var(--font-mono);
                font-size: 12px;
            }

            .stat-item {
                display: flex;
                justify-content: space-between;
                color: #94a3b8;
            }

            .stat-val {
                color: #38bdf8;
                font-weight: 500;
            }

            .elem-detail-title {
                font-family: var(--font-display);
                font-size: 18px;
                font-weight: 700;
                color: #fff;
                display: flex;
                align-items: center;
                gap: 10px;
                margin-bottom: 8px;
            }

            .elem-detail-desc {
                font-size: 12px;
                color: #94a3b8;
                line-height: 1.5;
            }

            .elem-properties {
                display: flex;
                flex-direction: column;
                gap: 6px;
                margin-top: 14px;
                font-size: 11px;
            }

            .elem-prop {
                display: flex;
                justify-content: space-between;
                padding: 4px 6px;
                background: rgba(255,255,255,0.02);
                border-radius: 4px;
                color: #64748b;
            }

            .elem-prop-val {
                color: #e2e8f0;
                font-weight: 500;
            }

            @media (max-width: 900px) {
                .main-content {
                    grid-template-columns: 1fr;
                    overflow-y: auto;
                }
                .dashboard {
                    height: auto;
                    max-height: none;
                    overflow: visible;
                }
            }
            "#
        </style>

        <div class="dashboard">
            <div class="header">
                <div class="title-area">
                    <h1>"NOITA SAND BOX"</h1>
                    <p>"WebGL & Rust WASM Physics Simulator"</p>
                </div>
                <div class="btn-group">
                    <button class="control-btn primary" on:click=handle_terrain>
                        "Generate Terrain"
                    </button>
                    <button class="control-btn" on:click=handle_clear>
                        "Clear Map"
                    </button>
                </div>
            </div>

            <div class="main-content">
                <div class="sidebar">
                    <div class="elem-group">
                        <h2>"Solids"</h2>
                        <div class="elem-grid">
                            {solids.into_iter().map(|el| {
                                let active = move || selected_element.get() == el;
                                view! {
                                    <button 
                                        class="elem-btn" 
                                        class:active=active
                                        on:click=move |_| set_selected_element.set(el)
                                    >
                                        <div class="color-dot" style=format!("background: {};", el.color_preview())></div>
                                        {el.name()}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="elem-group">
                        <h2>"Liquids"</h2>
                        <div class="elem-grid">
                            {liquids.into_iter().map(|el| {
                                let active = move || selected_element.get() == el;
                                view! {
                                    <button 
                                        class="elem-btn" 
                                        class:active=active
                                        on:click=move |_| set_selected_element.set(el)
                                    >
                                        <div class="color-dot" style=format!("background: {};", el.color_preview())></div>
                                        {el.name()}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="elem-group">
                        <h2>"Gases"</h2>
                        <div class="elem-grid">
                            {gases.into_iter().map(|el| {
                                let active = move || selected_element.get() == el;
                                view! {
                                    <button 
                                        class="elem-btn" 
                                        class:active=active
                                        on:click=move |_| set_selected_element.set(el)
                                    >
                                        <div class="color-dot" style=format!("background: {};", el.color_preview())></div>
                                        {el.name()}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="elem-group">
                        <h2>"Tools"</h2>
                        <button 
                            class="elem-btn" 
                            class:active=move || selected_element.get() == ElementType::Air
                            on:click=move |_| set_selected_element.set(ElementType::Air)
                        >
                            <div class="color-dot" style="background: #111;"></div>
                            "Eraser"
                        </button>
                    </div>
                </div>

                <div class="center-panel">
                    <div class="canvas-wrapper">
                        <canvas 
                            node_ref=canvas_ref
                            class="sim-canvas"
                            on:mousedown=on_mousedown
                            on:mousemove=on_mousemove
                            on:mouseup=on_mouseup
                            on:mouseleave=on_mouseleave
                            on:touchstart=on_touchstart
                            on:touchmove=on_touchmove
                            on:touchend=on_touchend
                        ></canvas>

                        {move || cursor_pos.get().map(|(x, y)| {
                            let dx_pct = (brush_size.get() as f64 * 2.0 + 1.0) / GRID_WIDTH as f64 * 100.0;
                            let dy_pct = (brush_size.get() as f64 * 2.0 + 1.0) / GRID_HEIGHT as f64 * 100.0;
                            view! {
                                <div 
                                    class="brush-preview" 
                                    style=format!(
                                        "left: {}%; top: {}%; width: {}%; height: {}%; transform: translate(-50%, -50%);", 
                                        x, y, dx_pct, dy_pct
                                    )
                                ></div>
                            }
                        })}
                    </div>

                    <div class="control-panel">
                        <div class="control-row">
                            <div class="btn-group">
                                <button 
                                    class="control-btn" 
                                    class:active=paused
                                    on:click=move |_| set_paused.update(|p| *p = !*p)
                                >
                                    {move || if paused.get() { "Resume" } else { "Pause" }}
                                </button>
                                <button class="control-btn" on:click=handle_step>
                                    "Step Frame"
                                </button>
                            </div>

                            <div class="slider-container">
                                <span>"Brush Size:"</span>
                                <div class="btn-group">
                                    {[1, 3, 5, 8, 12].into_iter().map(|sz| {
                                        let active = move || brush_size.get() == sz;
                                        view! {
                                            <button 
                                                class="control-btn"
                                                style="padding: 4px 10px; font-size: 11px;"
                                                class:active=active
                                                on:click=move |_| set_brush_size.set(sz)
                                            >
                                                {sz}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        </div>

                        <div class="control-row" style="border-top: 1px solid rgba(255,255,255,0.05); padding-top: 10px;">
                            <div class="slider-container">
                                <span>"Sim Speed:"</span>
                                <input 
                                    type="range" 
                                    min="1" 
                                    max="5" 
                                    prop:value=speed
                                    on:input=move |e| {
                                        if let Ok(val) = event_target_value(&e).parse::<i32>() {
                                            set_speed.set(val);
                                        }
                                    }
                                />
                                <span>{move || format!("{}x", speed.get())}</span>
                            </div>

                            <span style="font-size: 11px; color: #475569;">
                                "Drag mouse on canvas to draw elements."
                            </span>
                        </div>
                    </div>
                </div>

                <div class="sidebar info-panel">
                    <div class="card" style="flex: 1;">
                        <div class="card-title">"Element Inspector"</div>
                        <div class="elem-detail-title">
                            <div class="color-dot" style=format!("background: {};", selected_element.get().color_preview())></div>
                            {move || selected_element.get().name()}
                        </div>
                        <div class="elem-detail-desc">
                            {move || selected_element.get().description()}
                        </div>

                        <div class="elem-properties">
                            <div class="elem-prop">
                                <span>"Density:"</span>
                                <span class="elem-prop-val">{move || selected_element.get().density()}</span>
                            </div>
                            <div class="elem-prop">
                                <span>"State:"</span>
                                <span class="elem-prop-val">
                                    {move || {
                                        let el = selected_element.get();
                                        if el.is_static_solid() { "Static Solid" }
                                        else if el.is_falling_solid() { "Falling Solid" }
                                        else if el.is_liquid() { "Liquid" }
                                        else if el.is_gas() { "Gas" }
                                        else { "N/A" }
                                    }}
                                </span>
                            </div>
                            <div class="elem-prop">
                                <span>"Flammable:"</span>
                                <span class="elem-prop-val">
                                    {move || if selected_element.get().is_flammable() { "Yes" } else { "No" }}
                                </span>
                            </div>
                            <div class="elem-prop">
                                <span>"Acid Action:"</span>
                                <span class="elem-prop-val">
                                    {move || match selected_element.get() {
                                        ElementType::Acid => "Corrosive",
                                        ElementType::Stone => "Resistant",
                                        ElementType::Air => "N/A",
                                        _ => "Vulnerable",
                                    }}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div class="card">
                        <div class="card-title">"Performance Stats"</div>
                        <div class="stat-grid">
                            <div class="stat-item">
                                <span>"FPS:"</span>
                                <span class="stat-val">{fps}</span>
                            </div>
                            <div class="stat-item">
                                <span>"Active Cells:"</span>
                                <span class="stat-val">{particles}</span>
                            </div>
                            <div class="stat-item">
                                <span>"Grid Resolution:"</span>
                                <span class="stat-val">"240 x 180"</span>
                            </div>
                            <div class="stat-item">
                                <span>"Renderer Context:"</span>
                                <span class="stat-val" style="color: #10b981;">"WebGL2"</span>
                            </div>
                        </div>
                    </div>
                </div>

            </div>
        </div>
    }
}
