# ray-tracer-webgl

Rust/WASM + WebGL2 ray tracer based off of Peter Shirley's *Ray Tracing in One Weekend* series. I initially started this project as a software ray tracer running on Rust/WASM alone, but the render times that I experienced were so frustratingly slow that I quickly looked into implementing a hardware ray tracer that could take better advantage of the GPU's parallelization power. Once I switched to using WebGL2, render times went from around 1-6 minutes for a decent render to less than a second, and I was able to implement some realtime ray tracing elements like moving the camera, etc. by averaging many low-sample frames together rather than calculating them all at once.

One nice aspect of using Rust/WASM as the WebGL wrapper here is that Rust code is highly customizable, making it possible to create GLSL-like data structures like `vec3`s that respond to arithmetic operators in the same you'd expect a GLSL shader program to. This means setting up state and passing `vec3` data to the GPU becomes more convenient than using plain JS. For example, adding two vectors together in JS requires code like this `addVec3([0, 0, 0], [1, 1, 1])`, whereas by implementing the `Add` trait for a `Vec3` struct in Rust, you can write code more like `Vec3(0., 0., 0.) + Vec3(1., 1., 1.)`, which is very similar to GLSL's: `vec3(0., 0., 0.) + vec3(1., 1., 1.)`.

## Gallery

![14](/images/14.png)
![3](/images/3.png)
![1](/images/1.png)
![4](/images/4.png)
![5](/images/5.png)
![6](/images/6.png)
![7](/images/7.png)
![8](/images/8.png)
![9](/images/9.png)
![10](/images/10.png)
![11](/images/11.png)
![12](/images/12.png)
![16](/images/16.png)
![17](/images/17.png)
![18](/images/18.png)
![19](/images/19.png)