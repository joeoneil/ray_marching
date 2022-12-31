use ray_marching::run;
use ray_marching::util::image::Video;

fn main() {
    pollster::block_on(run());
}
