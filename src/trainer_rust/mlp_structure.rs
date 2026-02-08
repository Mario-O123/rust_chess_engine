//IMPORTANT: DONT CHANGE THINGS HERE UNLESS MODEL CHANGES TOO BECAUSE THIS CODE GETS CALLED IN THE FINAL FORWARD TOO

use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::tensor::Tensor;
use burn::tensor::backend::Backend;

//commented lines here are often placeholders because i frequently tweaked number of hidden layers to see which performed best

#[derive(Module, Debug)]
pub struct MLP<B: Backend> {
    pub fc1: Linear<B>,
    pub fc2: Linear<B>,
    pub fc3: Linear<B>,
   // pub fc4: Linear<B>,
}

impl<B: Backend> MLP<B> {
    pub fn new(
        input_size: usize,
        hidden_layer1: usize,
        hidden_layer2: usize,
       // hidden_layer3:usize,
        device: &B::Device,
    ) -> Self {
        //the linear functions
        let fc1: Linear<B> = LinearConfig::new(input_size, hidden_layer1).init(device);
        let fc2: Linear<B> = LinearConfig::new(hidden_layer1, hidden_layer2).init(device);
        let fc3: Linear<B> = LinearConfig::new(hidden_layer2, 1).init(device);
       // let fc4: Linear<B> = LinearConfig::new(hidden_layer3, 1).init(device);

        Self { fc1, fc2, fc3}
    }
    //the activation function for the hidden layer
    fn sc_relu(x: Tensor<B, 2>) -> Tensor<B, 2> {
        let clipped = x.clamp(0.0, 1.0);
        return clipped.square();
    }
    //use only 2 dimensional vectors for mlp logic
    //here we do the forward function of our mlp where we do linear forwards and then our activation function to calculate a score 
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.fc1.forward(x);
        //let x = activation::relu(x);

        let x = Self::sc_relu(x); //Self::sc_relu(

        let x = self.fc2.forward(x);
        //let x = activation::relu(x);
        let x = Self::sc_relu(x);
       // 
        let x = self.fc3.forward(x);
        //let x = Self::sc_relu(x);

        //let x = self.fc4.forward(x);

        return x;
    }
}
