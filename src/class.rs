use crate::{Result, UsbError};
use crate::bus::UsbBus;
use crate::allocator::{InterfaceNumber, StringIndex};
use crate::descriptor::{DescriptorWriter, BosWriter};
use crate::control;
use crate::control_pipe::ControlPipe;
use crate::endpoint::EndpointAddress;

/// A trait for implementing USB classes.
///
/// All methods are optional callbacks that will be called by
/// [UsbBus::poll](crate::bus::UsbBus::poll)
pub trait UsbClass<B: UsbBus> {
    /// Called when a GET_DESCRIPTOR request is received for a configuration descriptor. When
    /// called, the implementation should write its interface, endpoint and any extra class
    /// descriptors into `writer`. The configuration descriptor itself will be written by
    /// [UsbDevice](crate::device::UsbDevice) and shouldn't be written by classes.
    ///
    /// # Errors
    ///
    /// Generally errors returned by `DescriptorWriter`. Implementors should propagate any errors
    /// using `?`.
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()>;

    /// Called when a GET_DESCRIPTOR request is received for a BOS descriptor.
    /// When called, the implementation should write its blobs such as capability
    /// descriptors into `writer`. The BOS descriptor itself will be written by
    /// [UsbDevice](crate::device::UsbDevice) and shouldn't be written by classes.
    fn get_bos_descriptors(&self, writer: &mut BosWriter) -> Result<()> {
        let _ = writer;
        Ok (())
    }

    /// Gets a class-specific string descriptor.
    ///
    /// Note: All string descriptor requests are passed to all classes in turn, so implementations
    /// should return [`None`] if an unknown index is requested.
    ///
    /// # Arguments
    ///
    /// * `index` - A string index allocated earlier with
    ///   [`UsbAllocator`](crate::bus::UsbAllocator).
    /// * `lang_id` - The language ID for the string to retrieve.
    fn get_string(&self, index: StringIndex, lang_id: u16) -> Option<&str> {
        let _ = (index, lang_id);

        None
    }

    /// Called after a USB reset after the bus reset sequence is complete.
    fn reset(&mut self) { }

    /// Called when the device enters the Configured state. This method must enable the endpoints
    /// associated with the default alternate setting of each interface, thereby activating the
    /// default alternate setting.
    ///
    /// If the class does not use interface alternate settings, it can just enable all of its
    /// endpoints directly in this method, but if it does, it is recommended to delegate to
    /// `self.set_alternate_setting(iface, 0);` for each interface instead of enabling endpoints
    /// directly.
    fn configure(&mut self);

    /// Activates the specified alternate setting for the specified interface. The method must
    /// enable all endpoints used by the alternate setting, and disable all other endpoints not used
    /// by it. Not required if the class does not use any interface alternate settings.
    ///
    /// Note: This method may be called for an interface number you didn't allocate, in which case
    /// you should return the `InvalidInterface` error.
    ///
    /// # Errors
    ///
    /// * [`InvalidInterface`](crate::UsbError::InvalidInterface) - The interface number is not
    ///   defined by this class.
    /// * [`InvalidAlternateSetting`](crate::UsbError::InvalidAlternateSetting) - The `alt_setting`
    ///   value is not valid for the interface.
    fn set_alternate_setting(&mut self, interface: InterfaceNumber, alt_setting: u8) -> Result<()>
    {
        let _ = interface;
        let _ = alt_setting;

        Err(UsbError::InvalidInterface)
    }

    /// Gets the current active alternate setting for the specified interface. The value may be
    /// returned based on internal class state, or the value set via `reset`/`set_alternate_setting`
    /// may simply be stored and returned.
    ///
    /// Note: This method may be called for an interface number you didn't allocate, in which case
    /// you should return the `InvalidInterface` error.
    ///
    /// # Errors
    ///
    /// * [`InvalidInterface`](crate::UsbError::InvalidInterface) - The interface number is not
    ///   defined by this class.
    fn get_alternate_setting(&self, interface: InterfaceNumber) -> Result<u8> {
        let _ = interface;

        Err(UsbError::InvalidInterface)
    }

    /// Called whenever the `UsbDevice` is polled.
    fn poll(&mut self) { }

    /// Called when a control request is received with direction HostToDevice.
    ///
    /// All requests are passed to classes in turn, which can choose to accept, ignore or report an
    /// error. Classes can even choose to override standard requests, but doing that is rarely
    /// necessary.
    ///
    /// See [`ControlOut`] for how to respond to the transfer.
    ///
    /// When implementing your own class, you should ignore any requests that are not meant for your
    /// class so that any other classes in the composite device can process them.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    /// * `xfer` - A handle to the transfer.
    fn control_out(&mut self, xfer: ControlOut<B>) {
        let _ = xfer;
    }

    /// Called when a control request is received with direction DeviceToHost.
    ///
    /// All requests are passed to classes in turn, which can choose to accept, ignore or report an
    /// error. Classes can even choose to override standard requests, but doing that is rarely
    /// necessary.
    ///
    /// See [`ControlIn`] for how to respond to the transfer.
    ///
    /// When implementing your own class, you should ignore any requests that are not meant for your
    /// class so that any other classes in the composite device can process them.
    ///
    /// # Arguments
    ///
    /// * `req` - The request from the SETUP packet.
    /// * `data` - Data to send in the DATA stage of the control transfer.
    fn control_in(&mut self, xfer: ControlIn<B>) {
        let _ = xfer;
    }

    /// Called when endpoint with address `addr` has received a SETUP packet. Implementing this
    /// shouldn't be necessary in most cases, but is provided for completeness' sake.
    ///
    /// Note: This method may be called for an endpoint address you didn't allocate, in which case
    /// you should ignore the event.
    fn endpoint_setup(&mut self, addr: EndpointAddress) {
        let _ = addr;
    }

    /// Called when endpoint with address `addr` has received data (OUT packet).
    ///
    /// Note: This method may be called for an endpoint address you didn't allocate, in which case
    /// you should ignore the event.
    fn endpoint_out(&mut self, addr: EndpointAddress) {
        let _ = addr;
    }

    /// Called when endpoint with address `addr` has completed transmitting data (IN packet).
    ///
    /// Note: This method may be called for an endpoint address you didn't allocate, in which case
    /// you should ignore the event.
    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
        let _ = addr;
    }
}

/// Handle for a control IN transfer. When implementing a class, use the methods of this object to
/// response to the transfer with either data or an error (STALL condition). To ignore the request
/// and pass it on to the next class, simply don't call any method.
pub struct ControlIn<'p, 'r, B: UsbBus> {
    pipe: &'p mut ControlPipe<B>,
    req: &'r control::Request,
}

impl<'p, 'r, B: UsbBus> ControlIn<'p, 'r,  B> {
    pub(crate) fn new(pipe: &'p mut ControlPipe<B>, req: &'r control::Request) -> Self {
        ControlIn { pipe, req }
    }

    /// Gets the request from the SETUP packet.
    pub fn request(&self) -> &control::Request {
        self.req
    }

    /// Accepts the transfer with the supplied buffer.
    pub fn accept_with(self, data: &[u8]) -> Result<()> {
        self.pipe.accept_in(|buf| {
            if data.len() > buf.len() {
                return Err(UsbError::BufferOverflow);
            }

            buf[..data.len()].copy_from_slice(data);

            Ok(data.len())
        })
    }

    /// Accepts the transfer with the supplied static buffer.
    /// This method is useful when you have a large static descriptor to send as one packet.
    pub fn accept_with_static(self, data: &'static [u8]) -> Result<()> {
        self.pipe.accept_in_static(data)
    }

    /// Accepts the transfer with a callback that can write to the internal buffer of the control
    /// pipe. Can be used to avoid an extra copy.
    pub fn accept(self, f: impl FnOnce(&mut [u8]) -> Result<usize>) -> Result<()> {
        self.pipe.accept_in(f)
    }

    /// Rejects the transfer by stalling the pipe.
    pub fn reject(self) -> Result<()> {
        self.pipe.reject()
    }
}

/// Handle for a control OUT transfer. When implementing a class, use the methods of this object to
/// response to the transfer with an ACT or an error (STALL condition). To ignore the request and
/// pass it on to the next class, simply don't call any method.
pub struct ControlOut<'p, 'r, B: UsbBus> {
    pipe: &'p mut ControlPipe<B>,
    req: &'r control::Request,
}

impl<'p, 'r, B: UsbBus> ControlOut<'p, 'r, B> {
    pub(crate) fn new(pipe: &'p mut ControlPipe<B>, req: &'r control::Request) -> Self {
        ControlOut { pipe, req }
    }

    /// Gets the request from the SETUP packet.
    pub fn request(&self) -> &control::Request {
        self.req
    }

    /// Gets the data from the data stage of the request. May be empty if there was no data stage.
    pub fn data(&self) -> &[u8] {
        self.pipe.data()
    }

    /// Accepts the transfer by succesfully responding to the status stage.
    pub fn accept(self) -> Result<()> {
        self.pipe.accept_out()
    }

    /// Rejects the transfer by stalling the pipe.
    pub fn reject(self) -> Result<()> {
        self.pipe.reject()
    }
}
