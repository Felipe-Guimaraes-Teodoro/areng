use glam::f32::*;

use vulkano::buffer::subbuffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::pipeline::layout::PipelineLayout;
use std::sync::Arc;


use crate::rvkp::init::Vk;
use crate::rvkp::shader::vs::PushConstantCameraData;

use crate::utils::random;

const UP: Vec3A = vec3a(0.0, 1.0, 0.0);
const SPEED: f32 = 0.5;
const SENSITIVITY: f32 = 0.1;

#[derive(Debug, Clone)]
pub struct Camera {
    pub proj: Mat4,
    pub view: Mat4,

    pub pos: Vec3A,
    target: Vec3A,
    direction: Vec3A,
    pub right: Vec3A,
    pub front: Vec3A,
    pub up: Vec3A,

    pitch: f32,
    yaw: f32,

    pub dt: f32,
    last_frame: f32,

    first_mouse: bool,
    last_x: f32,
    last_y: f32,

    keymap: Vec<bool>,
}

impl Camera {
    pub fn new() -> Self {
        let (pitch, yaw): (f32, f32) = (0.0, 270.0);
        let pos = vec3a(0.0, 0.0, -1.0);
        let target = vec3a(0.0, 0.0, -1.0);
        let mut direction = Vec3A::normalize(pos - target);
        direction.x = yaw.to_radians().cos() * pitch.to_radians().cos();
        direction.y = pitch.to_radians().sin();
        direction.z = yaw.to_radians().sin() * pitch.to_radians().cos();
        
        let right = Vec3A::normalize(Vec3A::cross(UP, direction));
        let up = Vec3A::cross(direction, right);
        let front = Vec3A::normalize(direction);

        let view = Mat4::look_at_rh(
            pos.into(),
            (pos + front).into(),
            up.into(),
        );

        let proj = Mat4::perspective_lh(
            70.0_f32.to_radians(),
            1.0, 
            0.1, 
            1000.0,
        );

        Self {
            proj,
            view, 

            pos,
            target,
            direction,
            right,
            front,
            up,
            
            pitch,
            yaw,

            dt: 0.0,
            last_frame: 0.0,

            first_mouse: true,
            last_x: 400.0,
            last_y: 400.0,

            keymap: Vec::from_iter((0..6).map(|_| {false})),
        }
    }

    pub fn update(&mut self) {
        self.dt = 0.0016;

        if self.keymap[0] {
            self.pos -= SPEED * self.dt * self.front;
        }
        if self.keymap[1] {
            self.pos -= SPEED * self.dt * Vec3A::cross(self.front, self.up);
        }
        if self.keymap[2] {
            self.pos += SPEED * self.dt * self.front;
        }
        if self.keymap[3] {
            self.pos += SPEED * self.dt * Vec3A::cross(self.front, self.up);
        }

        self.view = Mat4::look_at_rh(
            self.pos.into(),
            (self.pos + self.front).into(),
            self.up.into(),
        );
    }

    pub fn input(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                let action = match input.state {
                    winit::event::ElementState::Pressed => true,
                    winit::event::ElementState::Released => false,
                };

                match input.virtual_keycode {
                    Some(winit::event::VirtualKeyCode::W) => {
                        self.keymap[0] = action;
                    },
                    Some(winit::event::VirtualKeyCode::A) => {
                        self.keymap[1] = action;
                    },
                    Some(winit::event::VirtualKeyCode::S) => {
                        self.keymap[2] = action;
                    },
                    Some(winit::event::VirtualKeyCode::D) => {
                        self.keymap[3] = action;
                    },
                    _ => ()
                }
            }

            _ => (),
        }
    }

    pub fn mouse_callback(
        &mut self, 
        xpos: f32, 
        ypos: f32,
    ) {
        if self.first_mouse { 
            self.last_x = xpos;
            self.last_y = ypos;
            self.first_mouse = false;
        }

        let mut xoffs = xpos - self.last_x;
        let mut yoffs = self.last_y - ypos;

        self.last_x = xpos;
        self.last_y = ypos;

        xoffs *= SENSITIVITY;
        yoffs *= SENSITIVITY;

        self.yaw += xoffs;
        self.pitch += yoffs;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        } 
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.direction.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.direction.y = self.pitch.to_radians().sin();
        self.direction.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.front = Vec3A::normalize(self.direction);
    }


    // RENDERING //

    pub fn send_push_constants(
        &mut self, 
        mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>, 
        layout: &Arc<PipelineLayout> 
        ) ->  AutoCommandBufferBuilder<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>, Arc<StandardCommandBufferAllocator>>
    {
        builder
            .push_constants(layout.clone(), 0, PushConstantCameraData {
                proj: self.proj.to_cols_array_2d(),
                view: self.view.to_cols_array_2d(),
            })
            .unwrap();

        builder
    }
}

