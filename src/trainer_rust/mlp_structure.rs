use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::tensor::Tensor;
use burn::tensor::activation::relu;
use burn::tensor::backend::Backend;

#[derive(Module, Debug)]
pub struct MLP<B: Backend> {
    pub fc1: Linear<B>,
    pub fc2: Linear<B>,
    pub fc3: Linear<B>,
}

impl<B: Backend> MLP<B> {
    pub fn new(
        input_size: usize,
        hidden_layer1: usize,
        hidden_layer2: usize,
        device: &B::Device,
    ) -> Self {
        let fc1: Linear<B> = LinearConfig::new(input_size, hidden_layer1).init(device);
        let fc2: Linear<B> = LinearConfig::new(hidden_layer1, hidden_layer2).init(device);
        let fc3: Linear<B> = LinearConfig::new(hidden_layer2, 1).init(device);

        Self { fc1, fc2, fc3 }
    }

    fn sc_relu(x: Tensor<B, 2>) -> Tensor<B, 2> {
        let clipped = x.clamp(0.0, 1.0);
        return clipped.square();
    }
    //for future its probably better to switch D to just 2 since i only work with 2 Dimensional tensors here
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.fc1.forward(x);
        let x = Self::sc_relu(x); //Self::sc_relu(

        let x = self.fc2.forward(x);
        let x = Self::sc_relu(x);

        let x = self.fc3.forward(x);

        return x;
    }
}
//here we have to implement the Module train for our MLP so Rust know at compiletime that our MLP has a forward pass or something like that
//or i guess we dont ??
