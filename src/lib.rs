//#![feature(associated_const_equality)]
use core::any::TypeId;
use core::marker::PhantomData;

fn send<'a, PB: PacketBuffer<true, 12> + 'a>(buffer: PB) {}

trait PacketBuffer<const CONTIGUOUS: bool, const HDR_RSV: usize> {}
impl<
        const CONTIGUOUS: bool,
        const HDR_RSV: usize,
        T: PacketBuffer<CONTIGUOUS, HDR_RSV> + ?Sized,
    > PacketBuffer<CONTIGUOUS, HDR_RSV> for &T
{
}

struct PacketBufferEnd;
impl<const CONTIGUOUS: bool, const HDR_RSV: usize> PacketBuffer<CONTIGUOUS, HDR_RSV>
    for PacketBufferEnd
{
}
impl PacketBufferEnd {
    pub const fn new() -> Self {
        PacketBufferEnd
    }
}
const PACKET_BUFFER_END: PacketBufferEnd = PacketBufferEnd::new();

trait PacketSliceTy<'a> {}
struct MutablePacketSliceTy<'a>(&'a mut [u8]);
impl<'a> PacketSliceTy<'a> for MutablePacketSliceTy<'a> {}
struct ImmutablePacketSliceTy<'a>(&'a [u8]);
impl<'a> PacketSliceTy<'a> for ImmutablePacketSliceTy<'a> {}

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

// impl<'a, 'b, const CONTIGUOUS: bool, const HDR_RSV: usize, const NEW_HDR_RSV: usize, const NEXT_CONTIGUOUS: bool, const NEXT_HDR_RSV: usize, S: PacketSliceTy<'a>, N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b> From<PacketSlice<'a, 

impl<'a, 'b, const HDR_RSV: usize, S: PacketSliceTy<'a>> PacketBuffer<true, HDR_RSV>
    for PacketSlice<'a, 'b, true, HDR_RSV, true, 0, S, PacketBufferEnd>
{
}
// impl<'a, 'b, const HDR_RSV: usize, S: PacketSliceTy<'a>> PacketBuffer<false, HDR_RSV>
//     for PacketSlice<'a, 'b, true, HDR_RSV, true, 0, S, PacketBufferEnd>
// {
// }
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

// impl<
//         'a,
//     'b,
//     const CONTIGUOUS: bool,
//         const HDR_RSV: usize,
//         const NEXT_CONTIGUOUS: bool,
//         const NEXT_HDR_RSV: usize,
//         N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
//     >
//     PacketSlice<
//         'a,
//         'static,
//         CONTIGUOUS,
//         HDR_RSV,
//         NEXT_CONTIGUOUS,
//         NEXT_HDR_RSV,
//         ImmutablePacketSliceTy<'a>,
//         N,
//     >
// {
//     pub fn resize_mut<'s, const NEW_HDR_RSV: usize>(&'s self) -> &'a PacketSlice<
//         'a,
//         'static,
//         CONTIGUOUS,
//         NEW_HDR_RSV,
//         NEXT_CONTIGUOUS,
//         NEXT_HDR_RSV,
//         ImmutablePacketSliceTy<'a>,
//         N,
// 	> {
// 	const _: () = assert!(NEW_HDR_RSV <= HDR_RSV);
// 	unsafe { core::mem::transmute(self) }
//     }
// }

// trait ResizePacketBuffer<'s, const HDR_RSV: usize, const NEW_HDR_RSV: usize> {
//     type Target;
//     const HDR_RSV_CONSTR: usize;
//     // const HDR_RSV_CONSTR: () = assert!(false);

//     fn resize(&'s self) -> &'s Self::Target;
//     fn resize_mut(&'s mut self) -> &'s mut Self::Target;
// }

// impl<
//         'a,
//     'b,
//     's,
//     const CONTIGUOUS: bool,
//     const NEW_HDR_RSV: usize,
//         const HDR_RSV: usize,
//         const NEXT_CONTIGUOUS: bool,
//     const NEXT_HDR_RSV: usize,
//     P: PacketSliceTy<'a>,

//     N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
//     >
//     ResizePacketBuffer<'s, HDR_RSV, NEW_HDR_RSV>
//     for
//     PacketSlice<
//         'a,
//         'b,
//         CONTIGUOUS,
//         HDR_RSV,
//         NEXT_CONTIGUOUS,
//         NEXT_HDR_RSV,
//         P,
//         N,
// 	> {
// 	type Target = PacketSlice<
//         'a,
//         'b,
//         CONTIGUOUS,
//         NEW_HDR_RSV,
//         NEXT_CONTIGUOUS,
//         NEXT_HDR_RSV,
//         P,
//         N,
// 	    >;


// 	// const HDR_RSV_CONSTR: usize = {
// 	//     assert!(HDR_RSV >= NEW_HDR_RSV);
// 	//     0
// 	// };
// 	const HDR_RSV_CONSTR: usize = 0;

// 	fn resize(&'s self) -> &'s Self::Target {
// 	    const _: () = assert!(HDR_RSV >= NEW_HDR_RSV);
// 	    unsafe { core::mem::transmute(self) }
// 	}


// 	fn resize_mut(&'s mut self) -> &'s mut Self::Target {
// 	    unsafe { core::mem::transmute(self) }
// 	}
//     }

fn resize_packet_slice_mut<
    'a,
    'b,
    's,
    const CONTIGUOUS: bool,
    const NEW_HDR_RSV: usize,
    const HDR_RSV: usize,
    const NEXT_CONTIGUOUS: bool,
    const NEXT_HDR_RSV: usize,
    P: PacketSliceTy<'a>,
    N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
>(
    s: &'s mut PacketSlice<'a, 'b, CONTIGUOUS, HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, P, N>
) -> &'s mut PacketSlice<'a, 'b, CONTIGUOUS, NEW_HDR_RSV, NEXT_CONTIGUOUS, NEXT_HDR_RSV, P, N> {
    let _: () = assert!(HDR_RSV >= NEW_HDR_RSV);
    unsafe { core::mem::transmute(s) }
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
}

impl<const LEN: usize> PacketBuffer<false, 0>
    for PacketArr<'static, true, LEN, true, 0, PacketBufferEnd>
{
}

impl<
        'b,
        const LEN: usize,
        const NEXT_CONTIGUOUS: bool,
        const NEXT_HDR_RSV: usize,
        N: PacketBuffer<NEXT_CONTIGUOUS, NEXT_HDR_RSV> + 'b,
    > PacketBuffer<false, 0> for PacketArr<'b, false, LEN, NEXT_CONTIGUOUS, NEXT_HDR_RSV, N>
{
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

// // // Type 4
// // #[repr(C)]
// // struct AnyPacketBufferRef<'a, 'b> {
// //     _type: usize,
// //     _type_id: TypeId,
// //     _ptr: *const (),
// //     _pd: PhantomData<(&'a (), &'b ())>,
// // }

// //struct IPStack<const CONTIGUOUS: bool, const HDR_RSV: usize> {
// //
// //}

// //struct IPService<'s, const CONTIGUOUS: bool, const IP_HDR_RSV: usize, B: PacketBuffer> {
// //    ip: &'s IPStack<IP_HDR_RSV>,
// //    _b: PhantomData<B>,
// // }

// // impl<'s, const IP_HDR_RSV: usize, B: PacketBuffer> IPService<'s, true, IP_HDR_RSV, B> {
// //     // This can be moved to the the impl definition when
// //     // `associated_const_equality` lands.
// //     const TX_BUF_CONTIGUOUS: () = assert!(B::CONTIGUOUS);
// //     const TX_BUF_HDR_RSV_OK: () = assert!(B::HDR_RSV >= IP_HDR_RSV + 4);
// // }

// // impl<'s, const IP_HDR_RSV: usize, B: PacketBuffer> IPService<'s, false, IP_HDR_RSV, B> {
// // }

#[cfg(test)]
mod tests {
    use super::*;

    fn accept_dyn_pb(_pb: &mut dyn PacketBuffer<true, 0>) {}

    #[test]
    fn test_types() {
        let empty_pb = PacketBufferEnd::new();

        let mut packet_data_arr = [0_u8; 1500];
        let mut hdr_mut_pb = PacketSlice::<'_, '_, true, 32, true, 0, _, _>::from_slice_mut_end(
            &mut packet_data_arr,
        );

	// let hdr_mut_pb_recast: () = hdr_mut_pb;

	let hdr_mut_pb_resized: &mut PacketSlice::<'_, '_, true, 33, true, 0, _, _>  =
	    resize_packet_slice_mut(&mut hdr_mut_pb);

        let arr_next_pb: PacketArr<'_, false, 12, true, 33, &dyn PacketBuffer<true, 33>> =
            PacketArr::from_arr([0; 12], hdr_mut_pb_resized);
    }
}
