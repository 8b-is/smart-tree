//! A Tensor with autograd capabilities.

use crate::gpu;
use ndarray::{s, Array, ArrayD, Axis, Ix2, Ix3, IxDyn};
use rand::distributions::{Distribution, Uniform};
use std::cell::{Ref, RefCell};
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

pub static CPU_MATMUL_TIME_NS: AtomicU64 = AtomicU64::new(0);

type BackwardOp = Rc<dyn Fn(&Tensor)>;

/// Holds the actual tensor data, its gradient, and graph information.
#[derive(Default)]
pub struct TensorData {
    pub data: ArrayD<f32>,
    pub grad: Option<Tensor>,
    _backward: Option<BackwardOp>,
    _prev: Vec<Tensor>,
}

impl fmt::Debug for TensorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TensorData")
            .field("data", &self.data)
            .field("grad", &self.grad)
            .field(
                "_backward",
                &self._backward.as_ref().map(|_| "BackwardOp"),
            )
            .field("_prev", &self._prev)
            .finish()
    }
}

/// The public Tensor struct, which is a smart pointer to the underlying data.
/// Cloning a Tensor is cheap as it only copies the Rc pointer.
#[derive(Debug, Clone)]
pub struct Tensor {
    pub inner: Rc<RefCell<TensorData>>,
}

// Allow Tensors in a HashSet for topological sort by comparing pointers.
impl PartialEq for Tensor {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for Tensor {}
impl Hash for Tensor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.inner.as_ptr()).hash(state);
    }
}

impl Tensor {
    /// Creates a new tensor from raw data and a shape. This creates a leaf node in the graph.
    pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Self {
        let array_shape = IxDyn(&shape);
        let data = Array::from_shape_vec(array_shape, data)
            .unwrap_or_else(|e| panic!("Data size does not match shape: {}", e));
        Self::from_data(data)
    }

    /// Creates a tensor from an existing ndarray::ArrayD. This creates a leaf node.
    pub fn from_data(data: ArrayD<f32>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(TensorData {
                data,
                ..Default::default()
            })),
        }
    }

    /// Creates a new tensor of zeros with the given shape.
    pub fn zeros(shape: Vec<usize>) -> Self {
        let array_shape = IxDyn(&shape);
        Self::from_data(Array::zeros(array_shape))
    }

    /// Creates a new tensor of ones with the same shape as the provided tensor.
    pub fn ones_like(tensor: &Tensor) -> Self {
        Self::from_data(Array::ones(tensor.shape()))
    }

    /// Creates a new tensor with random values for weight initialization.
    pub fn rand(shape: Vec<usize>) -> Self {
        let num_elements: usize = shape.iter().product();
        let mut rng = rand::thread_rng();
        let dist = Uniform::new(-0.02, 0.02);
        let data_vec: Vec<f32> = (0..num_elements).map(|_| dist.sample(&mut rng)).collect();
        Self::new(data_vec, shape)
    }

    // --- Accessors ---
    pub fn shape(&self) -> &[usize] {
        // Cannot return Ref<'_, _> directly due to lifetime issues.
        // This is a common pattern with RefCell.
        let r = self.inner.borrow();
        let shape = r.data.shape();
        unsafe { std::mem::transmute(shape) }
    }
    pub fn data(&self) -> Ref<'_, ArrayD<f32>> {
        Ref::map(self.inner.borrow(), |d| &d.data)
    }
    pub fn grad(&self) -> Option<Tensor> {
        self.inner.borrow().grad.clone()
    }
    pub fn set_grad(&self, grad: Tensor) {
        self.inner.borrow_mut().grad = Some(grad);
    }
    fn add_grad(&self, grad: Tensor) {
        let mut inner = self.inner.borrow_mut();
        if let Some(existing_grad) = inner.grad.take() {
            // If a gradient already exists, add the new gradient to it.
            let new_grad_data = &*existing_grad.data() + &*grad.data();
            inner.grad = Some(Tensor::from_data(new_grad_data));
        } else {
            // Otherwise, just set the new gradient.
            inner.grad = Some(grad);
        }
    }

    // --- Autograd and Optimizer Methods ---

    /// Kicks off the backpropagation process from this tensor.
    pub fn backward(&self) {
        let mut topo: Vec<Tensor> = Vec::new();
        let mut visited: HashSet<Tensor> = HashSet::new();
        fn build_topo(node: &Tensor, visited: &mut HashSet<Tensor>, topo: &mut Vec<Tensor>) {
            if !visited.contains(node) {
                visited.insert(node.clone());
                for child in &node.inner.borrow()._prev {
                    build_topo(child, visited, topo);
                }
                topo.push(node.clone());
            }
        }
        build_topo(self, &mut visited, &mut topo);

        // Set gradient of the final tensor to 1.0
        self.set_grad(Tensor::ones_like(self));

        // Go backwards through the sorted list, apply the backward pass, and clear the graph.
        // Clearing the graph after each step is crucial to prevent memory leaks from Rc cycles.
        for node in topo.iter().rev() {
            // 1. Execute the backward operation for the current node.
            // The backward op for a node calculates the gradients for its inputs (_prev nodes).
            if let Some(backward_fn) = node.inner.borrow()._backward.clone() {
                if let Some(grad) = node.grad() {
                    backward_fn(&grad);
                }
            }

            // 2. Clear the graph structure for the current node to free memory.
            // We only clear intermediate nodes (those that have a _backward op).
            // Leaf nodes (parameters, inputs) are not cleared and their grads are preserved.
            if node.inner.borrow()._backward.is_some() {
                let mut inner = node.inner.borrow_mut();
                inner._prev.clear();
                inner._backward = None;
                inner.grad = None; // Gradients on intermediate nodes are no longer needed.
            }
        }
    }

    /// Clears the gradient of the tensor.
    pub fn zero_grad(&self) {
        self.inner.borrow_mut().grad = None;
    }

    /// Updates the tensor's data using its gradient (for optimizers).
    pub fn update(&self, lr: f32) {
        let grad_opt = self.inner.borrow().grad.clone();
        if let Some(grad) = grad_opt {
            let grad_data = grad.data();
            let update_data = &*grad_data * lr;
            let mut inner = self.inner.borrow_mut();
            inner.data = &inner.data - &update_data;
        }
    }

    // --- Graph-aware Operations ---

    /// Applies the ReLU activation function.
    pub fn relu(&self) -> Tensor {
        let out_data = self.data().mapv(|x| if x > 0.0 { x } else { 0.0 });
        let out = Tensor::from_data(out_data);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            let relu_grad = {
                // Scope the immutable borrow from .data() so it's dropped before add_grad()
                let self_data = self_clone.data();
                self_data.mapv(|x| if x > 0.0 { 1.0 } else { 0.0 })
            };
            self_clone.add_grad(Tensor::from_data(&*grad.data() * &relu_grad));
        }));
        out
    }

    /// Applies the exponential function element-wise.
    pub fn exp(&self) -> Tensor {
        let out_data = self.data().mapv(f32::exp);
        let out = Tensor::from_data(out_data);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        let out_weak = Rc::downgrade(&out.inner);
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            // d(exp(x))/dx = exp(x) = y
            if let Some(out_rc) = out_weak.upgrade() {
                let out_tensor = Tensor { inner: out_rc };
                self_clone.add_grad(Tensor::from_data(&*grad.data() * &*out_tensor.data()));
            }
        }));
        out
    }

    /// Applies the natural logarithm element-wise.
    pub fn log(&self) -> Tensor {
        let out_data = self.data().mapv(|v| v.ln());
        let out = Tensor::from_data(out_data);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            // d(log(x))/dx = 1/x
            // The Ref from .data() must be dropped before .add_grad() is called on the same tensor.
            let new_grad_data = { &*grad.data() / &*self_clone.data() };
            self_clone.add_grad(Tensor::from_data(new_grad_data));
        }));
        out
    }

    /// Applies the square root function element-wise.
    pub fn sqrt(&self) -> Tensor {
        let out_data = self.data().mapv(f32::sqrt);
        let out = Tensor::from_data(out_data);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        let out_weak = Rc::downgrade(&out.inner);
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            if let Some(out_rc) = out_weak.upgrade() {
                // d(sqrt(x))/dx = 1 / (2 * sqrt(x))
                let out_tensor = Tensor { inner: out_rc };
                let grad_data = &*grad.data() / (2.0 * &*out_tensor.data());
                self_clone.add_grad(Tensor::from_data(grad_data));
            }
        }));
        out
    }

    /// Sums all elements in the tensor, returning a scalar tensor.
    pub fn sum(&self) -> Tensor {
        let out_data = self.data().sum();
        let out = Tensor::new(vec![out_data], vec![1]);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        let self_shape = self.shape().to_vec();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            let grad_val = *grad.data().first().unwrap();
            let num_elements = self_shape.iter().product();
            let grad_to_add = Tensor::new(vec![grad_val; num_elements], self_shape.clone());
            self_clone.add_grad(grad_to_add);
        }));
        out
    }

    /// Sums elements of a tensor along an axis.
    pub fn sum_axis(&self, axis: usize, keep_dims: bool) -> Tensor {
        let ax = Axis(axis);
        let out_data = if keep_dims {
            self.data().sum_axis(ax).insert_axis(ax)
        } else {
            self.data().sum_axis(ax)
        };
        let out = Tensor::from_data(out_data);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        let self_shape = self.shape().to_vec();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            // The gradient needs to be broadcasted back to the original shape.
            let grad_data = if keep_dims {
                grad.data().clone()
            } else {
                grad.data().clone().insert_axis(Axis(axis))
            };
            let broadcasted_grad = grad_data.broadcast(IxDyn(&self_shape)).unwrap().to_owned();
            self_clone.add_grad(Tensor::from_data(broadcasted_grad));
        }));
        out
    }

    /// Calculates the mean of elements along an axis.
    pub fn mean_axis(&self, axis: usize, keep_dims: bool) -> Tensor {
        let n = self.shape()[axis] as f32;
        self.sum_axis(axis, keep_dims) / n
    }

    /// Calculates the mean of all elements, returning a scalar tensor.
    pub fn mean(&self) -> Tensor {
        let n = self.data().len() as f32;
        self.sum() / n
    }

    /// Calculates the variance of elements along an axis.
    pub fn var_axis(&self, axis: usize, keep_dims: bool) -> Tensor {
        let mean = self.mean_axis(axis, true); // keep_dims=true for broadcasting
        let x_minus_mean = self - &mean;
        let x_minus_mean_sq = &x_minus_mean * &x_minus_mean;
        x_minus_mean_sq.mean_axis(axis, keep_dims)
    }

    /// Performs matrix multiplication.
    pub fn matmul(&self, other: &Tensor) -> Tensor {
        let self_shape = self.shape();
        let other_shape = other.shape();

        // --- GPU Acceleration Path ---
        let gpu_out_data = if gpu::USE_GPU.load(std::sync::atomic::Ordering::Relaxed) {
            if let Some(gpu_context) = &*gpu::GPU_CONTEXT {
                let mut gpu_op_result: Option<Result<ArrayD<f32>, &'static str>> = None;

                if self_shape.len() == 2 && other_shape.len() == 2 {
                    gpu_op_result = Some(gpu::gpu_matmul_2d(
                        gpu_context, &self.data(), &other.data(),
                    ));
                } else if self_shape.len() == 3 && other_shape.len() == 3 {
                    gpu_op_result = Some(gpu::gpu_matmul_3d(
                        gpu_context, &self.data(), &other.data(),
                    ));
                } else if self_shape.len() == 3 && other_shape.len() == 2 {
                    let batch_size = self_shape[0];
                    let m = self_shape[1];
                    let k = self_shape[2];
                    let n = other_shape[1];
                    if k == other_shape[0] {
                        let self_reshaped = self.data().clone().into_shape((batch_size * m, k)).unwrap().into_dyn();
                        gpu_op_result = Some(
                            gpu::gpu_matmul_2d(gpu_context, &self_reshaped, &other.data())
                                .map(|res| res.into_shape((batch_size, m, n)).unwrap().into_dyn()),
                        );
                    }
                }

                match gpu_op_result {
                    Some(Ok(data)) => Some(data),
                    Some(Err(e)) => {
                        // Keep this warning as it indicates a runtime problem.
                        println!("WARN: GPU matmul for shapes {:?} x {:?} failed ('{}'), falling back to CPU.", self_shape, other_shape, e);
                        None
                    }
                    None => None, // This means no matching GPU kernel was found for the shape.
                }
            } else {
                None // This means GPU context failed to initialize.
            }
        } else {
            None
        };

        let out_data = if let Some(data) = gpu_out_data {
            data
        } else {
            // --- CPU Path ---
            // This path is taken if USE_GPU is false, or if a GPU operation
            // was attempted but failed or was not implemented for the given shapes.
            let start = std::time::Instant::now();
            let result = if self_shape.len() == 2 && other_shape.len() == 2 {
                let a = self.data().clone().into_dimensionality::<Ix2>().unwrap();
                let b = other.data().clone().into_dimensionality::<Ix2>().unwrap();
                a.dot(&b).into_dyn()
            } else if self_shape.len() == 3 && other_shape.len() == 3 {
                let batch_size = self_shape[0];
                if batch_size != other_shape[0] { panic!("Batch dimensions must be equal for batched matmul: {:?} and {:?}", self_shape, other_shape); }
                if self_shape[2] != other_shape[1] { panic!("Incompatible dimensions for batched matmul: {:?} and {:?}", self_shape, other_shape); }

                let self_arr_3d = self.data().clone().into_dimensionality::<Ix3>().unwrap();
                let other_arr_3d = other.data().clone().into_dimensionality::<Ix3>().unwrap();

                let mut result_slices = Vec::new();
                for i in 0..batch_size {
                    result_slices.push(self_arr_3d.slice(s![i, .., ..]).dot(&other_arr_3d.slice(s![i, .., ..])));
                }
                let views: Vec<_> = result_slices.iter().map(|a| a.view()).collect();
                ndarray::stack(Axis(0), &views).unwrap().into_dyn()
            } else if self_shape.len() == 3 && other_shape.len() == 2 {
                let batch_size = self_shape[0];
                let m = self_shape[1];
                let k = self_shape[2];
                let n = other_shape[1];
                if k != other_shape[0] { panic!("Incompatible dimensions for 3D x 2D matmul: {:?} and {:?}", self_shape, other_shape); }
                
                let self_reshaped = self.data().clone().into_shape((batch_size * m, k)).unwrap();
                let other_2d = other.data().clone().into_dimensionality::<Ix2>().unwrap();
                let result_2d = self_reshaped.dot(&other_2d);
                result_2d.into_shape((batch_size, m, n)).unwrap().into_dyn()
            } else {
                panic!("Matmul not implemented for shapes {:?} and {:?}", self_shape, other_shape);
            };
            CPU_MATMUL_TIME_NS.fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
            result
        };
        
        let out = Tensor::from_data(out_data);
        out.inner.borrow_mut()._prev = vec![self.clone(), other.clone()];

        let self_clone = self.clone();
        let other_clone = other.clone();
        let self_shape_len = self.shape().len();
        let other_shape_len = other.shape().len();

        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            if self_shape_len == 2 && other_shape_len == 2 {
                let grad_a = grad.matmul(&other_clone.transpose(0, 1));
                self_clone.add_grad(grad_a);
                let grad_b = self_clone.transpose(0, 1).matmul(grad);
                other_clone.add_grad(grad_b);
            } else if self_shape_len == 3 && other_shape_len == 3 {
                let grad_a = grad.matmul(&other_clone.transpose(1, 2));
                self_clone.add_grad(grad_a);
                let grad_b = self_clone.transpose(1, 2).matmul(grad);
                other_clone.add_grad(grad_b);
            } else if self_shape_len == 3 && other_shape_len == 2 {
                let grad_a = grad.matmul(&other_clone.transpose(0, 1));
                self_clone.add_grad(grad_a);
                let self_reshaped = self_clone.reshape(vec![self_clone.shape()[0] * self_clone.shape()[1], self_clone.shape()[2]]);
                let grad_reshaped = grad.reshape(vec![grad.shape()[0] * grad.shape()[1], grad.shape()[2]]);
                let grad_b = self_reshaped.transpose(0,1).matmul(&grad_reshaped);
                other_clone.add_grad(grad_b);
            }
        }));
        out
    }
    
    /// Reshapes the tensor. This will copy data if the tensor is not contiguous.
    pub fn reshape(&self, new_shape: Vec<usize>) -> Tensor {
        let original_shape = self.shape().to_vec();
        // Use `as_standard_layout()` to create a contiguous copy if the array has a
        // non-standard memory layout (e.g., after a transpose). Then reshape.
        let reshaped_data = self
            .data()
            .as_standard_layout()
            .into_owned()
            .into_shape(IxDyn(&new_shape))
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to reshape tensor from {:?} to {:?}: {}",
                    original_shape, new_shape, e
                )
            });
        let out = Tensor::from_data(reshaped_data);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            // The gradient must also be reshaped back, which might also require making it contiguous.
            let grad_reshaped_data = grad
                .data()
                .as_standard_layout()
                .into_owned()
                .into_shape(IxDyn(&original_shape))
                .unwrap();
            self_clone.add_grad(Tensor::from_data(grad_reshaped_data));
        }));
        out
    }

    /// Swaps two axes of the tensor.
    pub fn transpose(&self, axis1: usize, axis2: usize) -> Tensor {
        let data_ref = self.data();
        let mut view = data_ref.view();
        view.swap_axes(axis1, axis2);
        let out = Tensor::from_data(view.to_owned());
        out.inner.borrow_mut()._prev = vec![self.clone()];

        let self_clone = self.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            // The backward of a transpose is a transpose.
            let grad_data_ref = grad.data();
            let mut grad_view = grad_data_ref.view();
            grad_view.swap_axes(axis1, axis2);
            self_clone.add_grad(Tensor::from_data(grad_view.to_owned()));
        }));
        out
    }

    /// Gathers slices from a tensor along a specified axis.
    /// Autograd-aware version of ndarray's `select`.
    pub fn gather(&self, indices: &Tensor, axis: usize) -> Tensor {
        let self_data = self.data();
        let indices_data = indices.data();
        assert_eq!(
            indices_data.ndim(),
            1,
            "Gather indices must be a 1D tensor."
        );
        let axis_obj = Axis(axis);

        // Collect views of the slices specified by the indices.
        let slices: Vec<_> = indices_data
            .iter()
            .map(|&idx| self_data.index_axis(axis_obj, idx as usize))
            .collect();

        // Stack the slices along a new first axis to create the output tensor.
        let output_data = ndarray::stack(Axis(0), &slices).unwrap();
        let out = Tensor::from_data(output_data);
        out.inner.borrow_mut()._prev = vec![self.clone()]; // Grad only w.r.t self (weights)

        let self_clone = self.clone();
        let indices_clone = indices.clone();
        let self_shape = self.shape().to_vec();

        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            // Backward pass for gather is scatter-add.
            // We create a zero-gradient tensor with the shape of the original weights
            // and add the incoming gradients to the rows specified by the indices.
            let mut grad_to_add = ArrayD::zeros(IxDyn(&self_shape));
            let grad_data = grad.data();
            let indices_data = indices_clone.data();

            for (i, &index) in indices_data.iter().enumerate() {
                let idx = index as usize;
                let mut slice = grad_to_add.index_axis_mut(axis_obj, idx);
                let grad_slice = grad_data.index_axis(Axis(0), i);
                slice += &grad_slice;
            }
            self_clone.add_grad(Tensor::from_data(grad_to_add));
        }));
        out
    }

    /// Applies the softmax function.
    pub fn softmax(&self, axis: usize) -> Tensor {
        let exp_x = self.exp();
        let sum_exp_x = exp_x.sum_axis(axis, true); // keep_dims=true for broadcasting
        &exp_x / &sum_exp_x
    }

    /// Applies the log_softmax function using the numerically stable log-sum-exp trick.
    pub fn log_softmax(&self, axis: usize) -> Tensor {
        // Stable log_softmax implementation:
        // log_softmax(x_i) = x_i - log(sum(exp(x_j)))
        //                = x_i - (max(x) + log(sum(exp(x_j - max(x)))))
        // This is implemented as a primitive operation for stability and performance.

        let ax = Axis(axis);
        let self_data = self.data();

        // --- Forward Pass (using ndarray directly for stability) ---
        // 1. Find max for stability
        let max_val = self_data.map_axis(ax, |view| {
            view.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
        });
        let max_expanded = max_val.clone().insert_axis(ax);

        // 2. Compute log_sum_exp
        let shifted = &*self_data - &max_expanded;
        let exp_shifted = shifted.mapv(f32::exp);
        let sum_exp_shifted = exp_shifted.sum_axis(ax);
        let log_sum_exp = sum_exp_shifted.mapv(f32::ln) + max_val;

        // 3. Compute final log_softmax output
        let out_data = &*self_data - &log_sum_exp.insert_axis(ax);

        let out = Tensor::from_data(out_data);
        out.inner.borrow_mut()._prev = vec![self.clone()];

        // --- Backward Pass ---
        let self_clone = self.clone();
        let out_weak = Rc::downgrade(&out.inner);
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            if let Some(out_rc) = out_weak.upgrade() {
                // Gradient of log_softmax is: grad_x = grad_y - exp(y) * sum(grad_y)
                // where y = log_softmax(x).
                let out_tensor = Tensor { inner: out_rc };
                let grad_data = grad.data();
                let softmax_out = out_tensor.data().mapv(f32::exp); // exp(log_softmax(x)) = softmax(x)

                let sum_grad = grad_data.sum_axis(ax).insert_axis(ax);
                let grad_term = &softmax_out * &sum_grad;

                let grad_to_add = &*grad_data - &grad_term;
                self_clone.add_grad(Tensor::from_data(grad_to_add));
            }
        }));

        out
    }

    /// Computes the negative log likelihood loss.
    /// `self` is log_probs with shape `[N, C]`, `targets` has shape `[N]`.
    pub fn nll_loss(&self, targets: &Tensor) -> Tensor {
        let self_data = self.data();
        let targets_data = targets.data();
        let batch_size = self_data.shape()[0];
        assert_eq!(self_data.ndim(), 2, "nll_loss input must be 2D");
        assert_eq!(targets_data.ndim(), 1, "nll_loss targets must be 1D");

        let mut loss_vec = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            let target_idx = targets_data[i] as usize;
            let log_prob = self_data[[i, target_idx]];
            loss_vec.push(-log_prob);
        }
        let loss_tensor = Tensor::new(loss_vec, vec![batch_size]);

        // Now for the backward pass.
        let self_clone = self.clone();
        let targets_clone = targets.clone();
        let self_shape = self.shape().to_vec();

        loss_tensor.inner.borrow_mut()._prev = vec![self.clone(), targets.clone()];
        loss_tensor.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            let mut grad_probs = ArrayD::zeros(IxDyn(&self_shape));
            let grad_data = grad.data(); // This should be shape [batch_size]
            let targets_data = targets_clone.data();

            for i in 0..batch_size {
                let target_idx = targets_data[i] as usize;
                // The gradient is -1 at the target index, scaled by the incoming gradient.
                grad_probs[[i, target_idx]] = -grad_data[i];
            }
            self_clone.add_grad(Tensor::from_data(grad_probs));
            // No gradient for targets.
        }));

        loss_tensor.mean() // Return the mean loss over the batch.
    }
}

// --- Operator Overloads ---

// --- Operator Overloads for Tensor ---

/// Helper function to sum a gradient back to the shape of the original tensor
/// in case of broadcasting. This is a crucial part of the backward pass for
/// operations like bias-add.
fn sum_grad_to_shape(grad_data: ArrayD<f32>, target_shape: &[usize]) -> ArrayD<f32> {
    let grad_shape = grad_data.shape().to_vec();
    if grad_shape == target_shape {
        return grad_data;
    }

    let mut summed_grad = grad_data;
    let mut axes_to_sum = Vec::new();

    let grad_ndim = grad_shape.len();
    let target_ndim = target_shape.len();

    // Identify prepended axes to sum over (e.g. batch and sequence dimensions for a bias).
    for i in 0..(grad_ndim.saturating_sub(target_ndim)) {
        axes_to_sum.push(i);
    }

    // Identify broadcasted axes to sum over (where target dim was 1).
    for i in 0..target_ndim {
        let grad_dim_idx = i + (grad_ndim - target_ndim);
        if target_shape[i] == 1 && grad_shape[grad_dim_idx] > 1 {
            axes_to_sum.push(grad_dim_idx);
        }
    }

    // Sum over all identified axes, in reverse order to maintain correct indices as axes are removed.
    axes_to_sum.sort();
    for &axis in axes_to_sum.iter().rev() {
        summed_grad = summed_grad.sum_axis(Axis(axis));
    }

    // Final reshape to match target shape (e.g., from `[C]` to `[1, C]`)
    let summed_grad_shape = summed_grad.shape().to_vec();
    summed_grad
        .to_owned() // Ensure the array is in standard memory layout before reshaping.
        .into_shape(IxDyn(target_shape))
        .unwrap_or_else(|e| {
            panic!(
                "Failed to reshape summed grad from {:?} to {:?}: {}",
                summed_grad_shape,
                target_shape,
                e
            )
        })
}


// --- ADD ---
impl Add for &Tensor {
    type Output = Tensor;
    fn add(self, rhs: &Tensor) -> Self::Output {
        let out = Tensor::from_data(&*self.data() + &*rhs.data());
        out.inner.borrow_mut()._prev = vec![self.clone(), rhs.clone()];

        let self_clone = self.clone();
        let rhs_clone = rhs.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            let self_shape = self_clone.shape().to_vec();
            let rhs_shape = rhs_clone.shape().to_vec();

            let grad_for_self = sum_grad_to_shape(grad.data().clone(), &self_shape);
            self_clone.add_grad(Tensor::from_data(grad_for_self));

            let grad_for_rhs = sum_grad_to_shape(grad.data().clone(), &rhs_shape);
            rhs_clone.add_grad(Tensor::from_data(grad_for_rhs));
        }));
        out
    }
}
impl<'a> Add<&'a Tensor> for Tensor { type Output = Tensor; fn add(self, rhs: &'a Tensor) -> Tensor { &self + rhs } }
impl Add<f32> for &Tensor {
    type Output = Tensor;
    fn add(self, rhs: f32) -> Self::Output {
        let out = Tensor::from_data(&*self.data() + rhs);
        out.inner.borrow_mut()._prev = vec![self.clone()];
        let self_clone = self.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            self_clone.add_grad(grad.clone());
        }));
        out
    }
}
impl Add<f32> for Tensor { type Output = Tensor; fn add(self, rhs: f32) -> Tensor { &self + rhs } }

// --- SUB ---
impl Sub for &Tensor {
    type Output = Tensor;
    fn sub(self, rhs: &Tensor) -> Self::Output {
        let out = Tensor::from_data(&*self.data() - &*rhs.data());
        out.inner.borrow_mut()._prev = vec![self.clone(), rhs.clone()];
        let self_clone = self.clone();
        let rhs_clone = rhs.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            let self_shape = self_clone.shape().to_vec();
            let rhs_shape = rhs_clone.shape().to_vec();

            let grad_for_self = sum_grad_to_shape(grad.data().clone(), &self_shape);
            self_clone.add_grad(Tensor::from_data(grad_for_self));

            let neg_grad = &*grad.data() * -1.0;
            let grad_for_rhs = sum_grad_to_shape(neg_grad, &rhs_shape);
            rhs_clone.add_grad(Tensor::from_data(grad_for_rhs));
        }));
        out
    }
}
impl<'a> Sub<&'a Tensor> for Tensor { type Output = Tensor; fn sub(self, rhs: &'a Tensor) -> Tensor { &self - rhs } }

// --- MUL ---
impl Mul for &Tensor {
    type Output = Tensor;
    fn mul(self, rhs: &Tensor) -> Self::Output {
        let out = Tensor::from_data(&*self.data() * &*rhs.data());
        out.inner.borrow_mut()._prev = vec![self.clone(), rhs.clone()];
        let self_clone = self.clone();
        let rhs_clone = rhs.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            let self_shape = self_clone.shape().to_vec();
            let rhs_shape = rhs_clone.shape().to_vec();

            let grad_for_self_data = { &*grad.data() * &*rhs_clone.data() };
            self_clone.add_grad(Tensor::from_data(sum_grad_to_shape(
                grad_for_self_data,
                &self_shape,
            )));

            let grad_for_rhs_data = { &*grad.data() * &*self_clone.data() };
            rhs_clone.add_grad(Tensor::from_data(sum_grad_to_shape(
                grad_for_rhs_data,
                &rhs_shape,
            )));
        }));
        out
    }
}
impl<'a> Mul<&'a Tensor> for Tensor { type Output = Tensor; fn mul(self, rhs: &'a Tensor) -> Tensor { &self * rhs } }

// --- DIV ---
impl Div for &Tensor {
    type Output = Tensor;
    fn div(self, rhs: &Tensor) -> Self::Output {
        let out = Tensor::from_data(&*self.data() / &*rhs.data());
        out.inner.borrow_mut()._prev = vec![self.clone(), rhs.clone()];
        let self_clone = self.clone();
        let rhs_clone = rhs.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            let self_shape = self_clone.shape().to_vec();
            let rhs_shape = rhs_clone.shape().to_vec();

            let grad_for_self_data = { &*grad.data() / &*rhs_clone.data() };
            self_clone.add_grad(Tensor::from_data(sum_grad_to_shape(
                grad_for_self_data,
                &self_shape,
            )));

            let grad_for_rhs_data = {
                let rhs_d = rhs_clone.data();
                &*grad.data() * (-&*self_clone.data() / (&*rhs_d * &*rhs_d))
            };
            rhs_clone.add_grad(Tensor::from_data(sum_grad_to_shape(
                grad_for_rhs_data,
                &rhs_shape,
            )));
        }));
        out
    }
}
impl<'a> Div<&'a Tensor> for Tensor { type Output = Tensor; fn div(self, rhs: &'a Tensor) -> Tensor { &self / rhs } }
impl Div<f32> for &Tensor {
    type Output = Tensor;
    fn div(self, rhs: f32) -> Self::Output {
        let out = Tensor::from_data(&*self.data() / rhs);
        out.inner.borrow_mut()._prev = vec![self.clone()];
        let self_clone = self.clone();
        out.inner.borrow_mut()._backward = Some(Rc::new(move |grad: &Tensor| {
            self_clone.add_grad(grad / rhs);
        }));
        out
    }
}
impl Div<f32> for Tensor { type Output = Tensor; fn div(self, rhs: f32) -> Tensor { &self / rhs } }

impl fmt::Display for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor(shape: {:?})\n{}", self.shape(), self.data())
    }
}
