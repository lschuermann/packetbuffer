use core::any::TypeId;
use core::marker::PhantomData;

/// Base PacketBuffer trait. Implemented by all elements of a PacketBuffer chain
/// (linked list). Generic over whether it is `CONTIGUOUS`, its guaranteed
/// headroom reservation `HDR_RSV`, and the list's tail type.
trait PacketBuffer<const CONTIGUOUS: bool, const HDR_RSV: usize> {
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
trait PacketSliceTy<'a> {
    fn len(&self) -> usize;
}

/// Mutable `PacketSliceTy`.
struct MutablePacketSliceTy<'a>(&'a mut [u8]);
impl<'a> PacketSliceTy<'a> for MutablePacketSliceTy<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Immutable `PacketSliceTy`.
struct ImmutablePacketSliceTy<'a>(&'a [u8]);
impl<'a> PacketSliceTy<'a> for ImmutablePacketSliceTy<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// `PacketBuffer` slice element. Must be generic over the `CONTIGUOUS` and
/// `HDR_RSV` const generic attributes of the _next_ type for Rust reasons.
struct PacketSlice<
    'a,
    'b,
    const CONTIGUOUS: bool,
    const HDR_RSV: usize,
    const NEXT_CONTIGUOUS: bool,
    const NEXT_HDR_RSV: usize,
    S: PacketSliceTy<'a>,
    N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
> {
    slice: S,
    next: N,
    _pd: PhantomData<(&'a (), &'b ())>,
}

/// If the next element in the `PacketBuffer` chain is the end
/// element, mark this slice as contiguous.
impl<'a, 'b, const HDR_RSV: usize, S: PacketSliceTy<'a>> PacketBuffer<true, HDR_RSV>
    for PacketSlice<'a, 'b, true, HDR_RSV, true, 0, S, PacketBufferEnd>
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
        'a,
        'b,
        const HDR_RSV: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        S: PacketSliceTy<'a>,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
    > PacketBuffer<false, HDR_RSV>
    for PacketSlice<'a, 'b, false, HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, S, N>
{
    fn len(&self) -> usize {
        self.slice.len() + self.next.len()
    }
}

impl<'a, const HDR_RSV: usize>
    PacketSlice<'a, 'static, true, HDR_RSV, true, 0, ImmutablePacketSliceTy<'a>, PacketBufferEnd>
{
    pub fn from_slice_end(slice: &'a [u8]) -> Self {
        PacketSlice {
            slice: ImmutablePacketSliceTy(slice),
            next: PacketBufferEnd::new(),
            _pd: PhantomData,
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
        'a,
        'b,
        const HDR_RSV: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
    >
    PacketSlice<
        'a,
        'static,
        false,
        HDR_RSV,
        NEXT_CONTIGUOUS,
        NEXT_HDR_RSV,
        ImmutablePacketSliceTy<'a>,
        N,
    >
{
    pub fn from_slice(slice: &'a [u8], next: N) -> Self {
        PacketSlice {
            slice: ImmutablePacketSliceTy(slice),
            next,
            _pd: PhantomData,
        }
    }
}

impl<'a, const HDR_RSV: usize>
    PacketSlice<'a, 'static, true, HDR_RSV, true, 0, MutablePacketSliceTy<'a>, PacketBufferEnd>
{
    pub fn from_slice_mut_end(slice: &'a mut [u8]) -> Self {
        PacketSlice {
            slice: MutablePacketSliceTy(slice),
            next: PacketBufferEnd::new(),
            _pd: PhantomData,
        }
    }
}

impl<
        'a,
        'b,
        const HDR_RSV: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
    >
    PacketSlice<
        'a,
        'static,
        false,
        HDR_RSV,
        NEXT_CONTIGUOUS,
        NEXT_HDR_RSV,
        MutablePacketSliceTy<'a>,
        N,
    >
{
    pub fn from_slice_mut(slice: &'a mut [u8], next: N) -> Self {
        PacketSlice {
            slice: MutablePacketSliceTy(slice),
            next,
            _pd: PhantomData,
        }
    }
}

struct PacketArr<
    'b,
    const CONTIGUOUS: bool,
    const LEN: usize,
    const NEXT_CONTIGUOUS: bool,
    const NEXT_HDR_RSV: usize,
    N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
> {
    arr: [u8; LEN],
    next: N,
    _pd: PhantomData<&'b ()>,
}

impl<const LEN: usize> PacketBuffer<true, 0>
    for PacketArr<'static, true, LEN, true, 0, PacketBufferEnd>
{
    fn len(&self) -> usize {
        self.arr.len() + self.next.len()
    }
}

impl<const LEN: usize> PacketBuffer<false, 0>
    for PacketArr<'static, true, LEN, true, 0, PacketBufferEnd>
{
    fn len(&self) -> usize {
        self.arr.len() + self.next.len()
    }
}

impl<
        'b,
        const LEN: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
    > PacketBuffer<false, 0> for PacketArr<'b, false, LEN, NEXT_CONTIGUOUS, NEXT_HDR_RSV, N>
{
    fn len(&self) -> usize {
        self.arr.len() + self.next.len()
    }
}

impl<const LEN: usize> PacketArr<'static, true, LEN, true, 0, PacketBufferEnd> {
    pub fn from_arr_end(arr: [u8; LEN]) -> Self {
        PacketArr {
            arr,
            next: PacketBufferEnd::new(),
            _pd: PhantomData,
        }
    }
}

impl<
        'b,
        const LEN: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
    > PacketArr<'b, false, LEN, NEXT_CONTIGUOUS, NEXT_HDR_RSV, N>
{
    pub fn from_arr(arr: [u8; LEN], next: N) -> Self {
        PacketArr {
            arr,
            next,
            _pd: PhantomData,
        }
    }
}

impl<
        'b,
        const CONTIGUOUS: bool,
        const LEN: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
    > PacketArr<'b, CONTIGUOUS, LEN, NEXT_CONTIGUOUS, NEXT_HDR_RSV, N>
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

struct BufferTypeCapture<
    const CONTIGUOUS: bool,
    const HDR_RSV: usize,
    T: PacketBuffer<CONTIGUOUS, HDR_RSV>,
> {
    state: core::cell::Cell<bool>,
    _pd: T,
}

impl<const CONTIGUOUS: bool, const HDR_RSV: usize, T: PacketBuffer<CONTIGUOUS, HDR_RSV>>
    BufferTypeCapture<CONTIGUOUS, HDR_RSV, T>
{
    pub fn capture(
        &self,
        buf: &'static mut T,
    ) -> Option<&'static mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV>> {
        if self.state.get() {
            // Already captured a type
            None
        } else {
            self.state.set(true);
            Some(buf)
        }
    }

    pub fn restore(
        &self,
        dyn_buf: &'static mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV>,
    ) -> Option<&'static mut T> {
        // TODO: how can we make sure that this buffer is of an
        // identical type to the one we passed down previously?
        if self.state.get() {
            Some(unsafe {
                std::mem::transmute::<*mut (), &'static mut T>(
                    dyn_buf as *mut dyn PacketBuffer<CONTIGUOUS, HDR_RSV> as *mut (),
                )
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types() {
        // fn send<'a, PB: PacketBuffer<true, 12> + 'a>(buffer: PB) {}
        fn accept_dyn_pb(_pb: &mut dyn PacketBuffer<true, 0>) {}

        let empty_pb = PacketBufferEnd::new();

        let mut packet_data_arr = [0_u8; 1500];
        let mut hdr_mut_pb = PacketSlice::<'_, '_, true, 32, true, 0, _, _>::from_slice_mut_end(
            &mut packet_data_arr,
        );

        let hdr_mut_pb_resized: &mut dyn PacketBuffer<true, 16> =
            shrink_packet_buffer_headroom(&mut hdr_mut_pb);

        // let arr_next_pb: PacketArr<'_, false, 12, true, 16, &dyn PacketBuffer<true, 16>> =
        //     PacketArr::from_arr([0; 12], hdr_mut_pb_resized);
    }

    struct ImANetworkLayer<const CONTIGUOUS: bool, const HEADROOM: usize> {
        higher_layer_adaptors:
            [Option<&'static dyn ThisIsAHigherLayerAdaptor<CONTIGUOUS, HEADROOM>>; 1],
    }

    impl<const CONTIGUOUS: bool, const HEADROOM: usize> ImANetworkLayer<CONTIGUOUS, HEADROOM> {
        fn dispatch_buffer(&self, buffer: &'static mut dyn PacketBuffer<CONTIGUOUS, HEADROOM>) {
            self.higher_layer_adaptors[0]
                .unwrap()
                .pass_buffer_back(buffer);
        }
    }

    trait ThisIsAHigherLayerAdaptor<const CONTIGUOUS: bool, const HEADROOM: usize> {
        fn pass_buffer_back(&self, buffer: &'static mut dyn PacketBuffer<CONTIGUOUS, HEADROOM>);
    }

    struct ImAHigherLayerAdaptor<
        const CONTIGUOUS: bool,
        const NETWORK_LAYER_HEADROOM: usize,
        const HIGHER_LAYER_HEADROOM: usize,
        HPB: PacketBuffer<CONTIGUOUS, HIGHER_LAYER_HEADROOM> + 'static,
    > {
        network_layer: &'static ImANetworkLayer<CONTIGUOUS, NETWORK_LAYER_HEADROOM>,
        type_capture: BufferTypeCapture<CONTIGUOUS, HIGHER_LAYER_HEADROOM, HPB>,
        _pd: PhantomData<HPB>,
    }

    impl<
            const CONTIGUOUS: bool,
            const NETWORK_LAYER_HEADROOM: usize,
            const HIGHER_LAYER_HEADROOM: usize,
            HPB: PacketBuffer<CONTIGUOUS, HIGHER_LAYER_HEADROOM> + 'static,
        > ThisIsAHigherLayerAdaptor<CONTIGUOUS, NETWORK_LAYER_HEADROOM>
        for ImAHigherLayerAdaptor<CONTIGUOUS, NETWORK_LAYER_HEADROOM, HIGHER_LAYER_HEADROOM, HPB>
    {
        fn pass_buffer_back(
            &self,
            buffer: &'static mut dyn PacketBuffer<CONTIGUOUS, NETWORK_LAYER_HEADROOM>,
        ) {
            let original_buf: &'static mut HPB = self
                .type_capture
                .restore(restore_packet_buffer_headroom(buffer).unwrap())
                .unwrap();
        }
    }

    impl<
            const CONTIGUOUS: bool,
            const NETWORK_LAYER_HEADROOM: usize,
            const HIGHER_LAYER_HEADROOM: usize,
            HPB: PacketBuffer<CONTIGUOUS, HIGHER_LAYER_HEADROOM> + 'static,
        > ImAHigherLayerAdaptor<CONTIGUOUS, NETWORK_LAYER_HEADROOM, HIGHER_LAYER_HEADROOM, HPB>
    {
        fn dispatch_buffer(&self, buf: &'static mut HPB) {
            let dyn_buf = self.type_capture.capture(buf).unwrap();
            self.network_layer
                .dispatch_buffer(shrink_packet_buffer_headroom(dyn_buf));
        }
    }
}
