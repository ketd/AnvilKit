use anvilkit_render::renderer::device::RenderDevice;

/// Clear the swapchain to a dark background (for main menu).
pub fn clear_to_dark(
    _device: &RenderDevice,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Menu Clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.002,
                    g: 0.002,
                    b: 0.005,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        ..Default::default()
    });
}
