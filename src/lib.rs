use core::any::{Any, TypeId};
use core::marker::PhantomData;

/// Base PacketBuffer trait. Implemented by all elements of a PacketBuffer chain
/// (linked list). Generic over whether it is `CONTIGUOUS`, its guaranteed
/// headroom reservation `HDR_RSV`, and the list's tail type.
// Indicating the Any supertrait is important, to be able to use Any's
// downcast_ref method to convert a trait object of this type back
// into its original type.
trait PacketBuffer<const CONTIGUOUS: bool, const HDR_RSV: usize>: Any {
    fn len(&self) -> usize;
}

// impl<
//         const CONTIGUOUS: bool,
//         const HDR_RSV: usize,
//         T: PacketBuffer<CONTIGUOUS, HDR_RSV> + ?Sized,
//     > PacketBuffer<CONTIGUOUS, HDR_RSV> for &T
// {
//     type ShrunkOut<const NEW_HDR_RSV: usize> = T::ShrunkOut<NEW_HDR_RSV>;

//     fn len(&self) -> usize {
// 	(*self).len()
//     }

//     fn shrink_headroom<
// 	's,
// 	const NEW_HDR_RSV: usize,
// 	>(
// 	s: &'s mut Self,
//     ) -> &'s mut Self::ShrunkOut<NEW_HDR_RSV> {
// 	T::shrink_headroom::<NEW_HDR_RSV>(s)
//     }
// }

/// Dummy type denoting the end of a `PacketBuffer` chain. Cannot hold any data,
/// always contiguous, zero headroom reservation.
struct PacketBufferEnd;
impl PacketBuffer<true, 0> for PacketBufferEnd {
    fn len(&self) -> usize {
        0
    }
}
impl PacketBufferEnd {
    pub const fn new() -> Self {
        PacketBufferEnd
    }
}

// For convience, we provide a const instantiation of this type.
const PACKET_BUFFER_END: PacketBufferEnd = PacketBufferEnd::new();

/// Slice packet buffer trait. This allows us to implement only a single
/// `PacketSlice`, which is generic over a `PacketSliceTy` that is either
/// mutable or immutable.
trait PacketSliceTy {
    fn len(&self) -> usize;
}

/// Mutable `PacketSliceTy`.
struct MutablePacketSliceTy(&'static mut [u8]);
impl PacketSliceTy for MutablePacketSliceTy {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Immutable `PacketSliceTy`.
struct ImmutablePacketSliceTy(&'static [u8]);
impl PacketSliceTy for ImmutablePacketSliceTy {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// `PacketBuffer` slice element. Must be generic over the `CONTIGUOUS` and
/// `HDR_RSV` const generic attributes of the _next_ type for Rust reasons.
struct PacketSlice<
    const CONTIGUOUS: bool,
    const HDR_RSV: usize,
    const NEXT_CONTIGUOUS: bool,
    const NEXT_HDR_RSV: usize,
    S: PacketSliceTy + 'static,
    N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
> {
    slice: S,
    next: N,
}

/// If the next element in the `PacketBuffer` chain is the end
/// element, mark this slice as contiguous.
impl<const HDR_RSV: usize, S: PacketSliceTy> PacketBuffer<true, HDR_RSV>
    for PacketSlice<true, HDR_RSV, true, 0, S, PacketBufferEnd>
{
    fn len(&self) -> usize {
        self.slice.len() + self.next.len()
    }
}

// Otherwise, for any other element, only implement the non-contiguous
// cases.
/// Regardless of the next chain element, this `PacketSlice` always
/// implements the non-contiguous `PacketBuffer` interface.
impl<
        const HDR_RSV: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        S: PacketSliceTy + 'static,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
    > PacketBuffer<false, HDR_RSV>
    for PacketSlice<false, HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, S, N>
{
    fn len(&self) -> usize {
        self.slice.len() + self.next.len()
    }
}

impl<const HDR_RSV: usize>
    PacketSlice<true, HDR_RSV, true, 0, ImmutablePacketSliceTy, PacketBufferEnd>
{
    pub fn from_slice_end(slice: &'static [u8]) -> Self {
        PacketSlice {
            slice: ImmutablePacketSliceTy(slice),
            next: PacketBufferEnd::new(),
        }
    }
}

// trait ResizePacketBufferHeadroom<
// 	const CONTIGUOUS: bool,
// 	const HDR_RSV: usize,
// 	PB: PacketBuffer<CONTIGUOUS, HDR_RSV> + ?Sized
// > {
//     /// Support shrinking the packet slice headroom.
//     fn shrink_packet_slice_headroom<
// 	    'a,
// 	'b,
// 	's,
// 	// const CONTIGUOUS: bool,
// 	const NEW_HDR_RSV: usize,
// 	// const HDR_RSV: usize,
// 	const NEXT_CONTIGUOUS: bool,
// 	const NEXT_HDR_RSV: usize,
// 	P: PacketSliceTy<'a>,
// 	N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
// 	>(
// 	s: &'s mut PacketSlice<'a, 'b, CONTIGUOUS, HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, P, N>,
//     ) -> &'s mut PacketSlice<'a, 'b, CONTIGUOUS, NEW_HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, P, N> {
// 	let _: () = assert!(HDR_RSV >= NEW_HDR_RSV);
// 	unsafe { core::mem::transmute(s) }
//     }
// }

/// Support shrinking the packet slice headroom.
fn shrink_packet_buffer_headroom<
    's,
    const CONTIGUOUS: bool,
    const NEW_HDR_RSV: usize,
    const HDR_RSV: usize,
>(
    s: &'s mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV>,
) -> &'s mut dyn PacketBuffer<CONTIGUOUS, NEW_HDR_RSV> {
    let _: () = assert!(HDR_RSV >= NEW_HDR_RSV);
    unsafe { core::mem::transmute(s) }
}

/// Support shrinking the packet slice headroom.
fn restore_packet_buffer_headroom<
    's,
    const CONTIGUOUS: bool,
    const NEW_HDR_RSV: usize,
    const HDR_RSV: usize,
>(
    s: &'s mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV>,
) -> Option<&'s mut dyn PacketBuffer<CONTIGUOUS, NEW_HDR_RSV>> {
    // TODO: check that we don't go beyond the length!
    Some(unsafe { core::mem::transmute(s) })
}

/// Support the headroom up to the entire packetbuffer's length.
// fn restore_packet_slice_headroom<
//     'a,
//     'b,
//     's,
//     const CONTIGUOUS: bool,
//     const NEW_HDR_RSV: usize,
//     const HDR_RSV: usize,
//     const NEXT_CONTIGUOUS: bool,
//     const NEXT_HDR_RSV: usize,
//     P: PacketSliceTy<'a>,
//     N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
// >(
//     s: &'s mut PacketSlice<'a, 'b, CONTIGUOUS, HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, P, N>,
// ) -> Option<&'s mut PacketSlice<'a, 'b, CONTIGUOUS, NEW_HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, P, N>> {
//     if NEW_HDR_RSV <= s.len() {
// 	Some(unsafe { core::mem::transmute(s) })
//     } else {
// 	None
//     }
// }

impl<
        const HDR_RSV: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
    >
    PacketSlice<
        false,
        HDR_RSV,
        NEXT_CONTIGUOUS,
        NEXT_HDR_RSV,
        ImmutablePacketSliceTy,
        N,
    >
{
    pub fn from_slice(slice: &'static [u8], next: N) -> Self {
        PacketSlice {
            slice: ImmutablePacketSliceTy(slice),
            next,
        }
    }
}

impl<const HDR_RSV: usize>
    PacketSlice<true, HDR_RSV, true, 0, MutablePacketSliceTy, PacketBufferEnd>
{
    pub fn from_slice_mut_end(slice: &'static mut [u8]) -> Self {
        PacketSlice {
            slice: MutablePacketSliceTy(slice),
            next: PacketBufferEnd::new(),
        }
    }
}

impl<
        const HDR_RSV: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
    >
    PacketSlice<
        false,
        HDR_RSV,
        NEXT_CONTIGUOUS,
        NEXT_HDR_RSV,
        MutablePacketSliceTy,
        N,
    >
{
    pub fn from_slice_mut(slice: &'static mut [u8], next: N) -> Self {
        PacketSlice {
            slice: MutablePacketSliceTy(slice),
            next,
        }
    }
}

struct PacketArr<
    const CONTIGUOUS: bool,
    const LEN: usize,
    const NEXT_CONTIGUOUS: bool,
    const NEXT_HDR_RSV: usize,
    N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
> {
    arr: [u8; LEN],
    next: N,
}

impl<const LEN: usize> PacketBuffer<true, 0>
    for PacketArr<true, LEN, true, 0, PacketBufferEnd>
{
    fn len(&self) -> usize {
        self.arr.len() + self.next.len()
    }
}

impl<const LEN: usize> PacketBuffer<false, 0>
    for PacketArr<true, LEN, true, 0, PacketBufferEnd>
{
    fn len(&self) -> usize {
        self.arr.len() + self.next.len()
    }
}

impl<
        const LEN: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
    > PacketBuffer<false, 0> for PacketArr<false, LEN, NEXT_CONTIGUOUS, NEXT_HDR_RSV, N>
{
    fn len(&self) -> usize {
        self.arr.len() + self.next.len()
    }
}

impl<const LEN: usize> PacketArr<true, LEN, true, 0, PacketBufferEnd> {
    pub fn from_arr_end(arr: [u8; LEN]) -> Self {
        PacketArr {
            arr,
            next: PacketBufferEnd::new(),
        }
    }
}

impl<
        const LEN: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
    > PacketArr<false, LEN, NEXT_CONTIGUOUS, NEXT_HDR_RSV, N>
{
    pub fn from_arr(arr: [u8; LEN], next: N) -> Self {
        PacketArr {
            arr,
            next,
        }
    }
}

impl<
        const CONTIGUOUS: bool,
        const LEN: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV>,
    > PacketArr<CONTIGUOUS, LEN, NEXT_CONTIGUOUS, NEXT_HDR_RSV, N>
{
    pub fn into_inner(self) -> ([u8; LEN], N) {
        (self.arr, self.next)
    }

    pub fn inner(&self) -> &[u8; LEN] {
        &self.arr
    }

    pub fn inner_mut(&mut self) -> &[u8; LEN] {
        &mut self.arr
    }
}

// struct BufferTypeCapture<
//     const CONTIGUOUS: bool,
//     const HDR_RSV: usize,
//     T: PacketBuffer<CONTIGUOUS, HDR_RSV>,
// > {
//     state: core::cell::Cell<bool>,
//     _pd: T,
// }

// impl<const CONTIGUOUS: bool, const HDR_RSV: usize, T: PacketBuffer<CONTIGUOUS, HDR_RSV>>
//     BufferTypeCapture<CONTIGUOUS, HDR_RSV, T>
// {
//     pub fn capture(
//         &self,
//         buf: &'static mut T,
//     ) -> Option<&'static mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV>> {
//         if self.state.get() {
//             // Already captured a type
//             None
//         } else {
//             self.state.set(true);
//             Some(buf)
//         }
//     }

//     pub fn restore(
//         &self,
//         dyn_buf: &'static mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV>,
//     ) -> Option<&'static mut T> {
//         // TODO: how can we make sure that this buffer is of an
//         // identical type to the one we passed down previously?
//         if self.state.get() {
//             Some(unsafe {
//                 std::mem::transmute::<*mut (), &'static mut T>(
//                     dyn_buf as *mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV> as *mut (),
//                 )
//             })
//         } else {
//             None
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn test_types() {
        // fn send<'a, PB: PacketBuffer<true, 12> + 'a>(buffer: PB) {}
        fn accept_dyn_pb(_pb: &mut dyn PacketBuffer<true, 0>) {}

        let empty_pb = Box::leak(Box::new(PacketBufferEnd::new()));

        let mut packet_data_arr = Box::leak(Box::new([0_u8; 1500]));
        let mut hdr_mut_pb = PacketSlice::<true, 32, true, 0, _, _>::from_slice_mut_end(
            packet_data_arr,
        );

        let hdr_mut_pb_resized: &mut dyn PacketBuffer<true, 16> =
            shrink_packet_buffer_headroom(&mut hdr_mut_pb);

        // let arr_next_pb: PacketArr<false, 12, true, 16, &dyn PacketBuffer<true, 16>> =
        //     PacketArr::from_arr([0; 12], hdr_mut_pb_resized);
    }

    struct ImANetworkLayer<'a, const CONTIGUOUS: bool, const HEADROOM: usize> {
        higher_layer_adaptors:
            [Cell<Option<&'a dyn ThisIsAHigherLayerAdaptor<CONTIGUOUS, HEADROOM>>>; 1],
    }

    impl<const CONTIGUOUS: bool, const HEADROOM: usize> ImANetworkLayer<'_, CONTIGUOUS, HEADROOM> {
        fn dispatch_buffer(&self, buffer: &'static mut dyn PacketBuffer<CONTIGUOUS, HEADROOM>) {
            self.higher_layer_adaptors[0]
                .get()
                .unwrap()
                .pass_buffer_back(buffer);
        }
    }

    trait ThisIsAHigherLayerAdaptor<const CONTIGUOUS: bool, const HEADROOM: usize> {
        fn pass_buffer_back(&self, buffer: &'static mut dyn PacketBuffer<CONTIGUOUS, HEADROOM>);
    }

    struct ImAHigherLayerAdaptor<
	    'a,
        const CONTIGUOUS: bool,
        const NETWORK_LAYER_HEADROOM: usize,
        const HIGHER_LAYER_HEADROOM: usize,
        HPB: PacketBuffer<CONTIGUOUS, HIGHER_LAYER_HEADROOM> + 'static,
    > {
        network_layer: &'a ImANetworkLayer<'a, CONTIGUOUS, NETWORK_LAYER_HEADROOM>,
        // type_capture: BufferTypeCapture<CONTIGUOUS, HIGHER_LAYER_HEADROOM, HPB>,
        _pd: PhantomData<HPB>,
    }

    impl<'a,
            const CONTIGUOUS: bool,
            const NETWORK_LAYER_HEADROOM: usize,
            const HIGHER_LAYER_HEADROOM: usize,
            HPB: PacketBuffer<CONTIGUOUS, HIGHER_LAYER_HEADROOM> + 'static,
        > ThisIsAHigherLayerAdaptor<CONTIGUOUS, NETWORK_LAYER_HEADROOM>
        for ImAHigherLayerAdaptor<'a, CONTIGUOUS, NETWORK_LAYER_HEADROOM, HIGHER_LAYER_HEADROOM, HPB>
    {
        fn pass_buffer_back(
            &self,
            buffer: &'static mut dyn PacketBuffer<CONTIGUOUS, NETWORK_LAYER_HEADROOM>,
        ) {
	    let any_buffer: &'static mut dyn Any = buffer as _;
	    match any_buffer.downcast_mut::<HPB>() {
		Some(original_buffer) => {
		    let _: &'static mut HPB = original_buffer;
		    println!("Received back valid buffer type!");
		},
		None => {
		    panic!("Invalid buffer type passed back!");
		},
	    }
        }
    }

    impl<
	    'a,
            const CONTIGUOUS: bool,
            const NETWORK_LAYER_HEADROOM: usize,
            const HIGHER_LAYER_HEADROOM: usize,
            HPB: PacketBuffer<CONTIGUOUS, HIGHER_LAYER_HEADROOM> + 'static,
        > ImAHigherLayerAdaptor<'a, CONTIGUOUS, NETWORK_LAYER_HEADROOM, HIGHER_LAYER_HEADROOM, HPB>
    {
        fn dispatch_buffer(&self, buf: &'static mut HPB) {
            // let dyn_buf = self.type_capture.capture(buf).unwrap();
            self.network_layer
                .dispatch_buffer(shrink_packet_buffer_headroom(buf));
        }
    }

    #[test]
    fn test_layers() {
	let mut network_layer = ImANetworkLayer {
	    higher_layer_adaptors: [Cell::new(None)],
	};

	let adaptor: ImAHigherLayerAdaptor<'_, true, 0, 0, PacketBufferEnd> = ImAHigherLayerAdaptor {
	    network_layer: &network_layer,
	    _pd: PhantomData,
	};

	network_layer.higher_layer_adaptors[0].set(Some(&adaptor));

	adaptor.dispatch_buffer(Box::leak(Box::new(PacketBufferEnd)));
    }
}
