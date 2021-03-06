use ffi;
use std::fmt;
use std::ptr;
use std::mem;
use libc::c_int;
use foreign_types::ForeignTypeRef;

use {cvt, cvt_n, cvt_p};
use bn::{BigNum, BigNumRef};
use error::ErrorStack;

/// Type of encryption padding to use.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Padding(c_int);

impl Padding {
    pub fn from_raw(value: c_int) -> Padding {
        Padding(value)
    }

    pub fn as_raw(&self) -> c_int {
        self.0
    }

    pub const NONE: Padding = Padding(ffi::RSA_NO_PADDING);
    pub const PKCS1: Padding = Padding(ffi::RSA_PKCS1_PADDING);
    pub const PKCS1_OAEP: Padding = Padding(ffi::RSA_PKCS1_OAEP_PADDING);
}

foreign_type_and_impl_send_sync! {
    type CType = ffi::RSA;
    fn drop = ffi::RSA_free;

    pub struct Rsa;
    pub struct RsaRef;
}

impl RsaRef {
    // FIXME these need to specify output format
    private_key_to_pem!(ffi::PEM_write_bio_RSAPrivateKey);
    public_key_to_pem!(ffi::PEM_write_bio_RSA_PUBKEY);

    private_key_to_der!(ffi::i2d_RSAPrivateKey);
    public_key_to_der!(ffi::i2d_RSA_PUBKEY);

    to_der_inner!(
        /// Serializes the public key to DER-encoded PKCS#1.
        public_key_to_der_pkcs1,
        ffi::i2d_RSAPublicKey
    );

    pub fn size(&self) -> u32 {
        unsafe {
            assert!(self.n().is_some());
            ffi::RSA_size(self.as_ptr()) as u32
        }
    }

    /// Decrypts data using the private key, returning the number of decrypted bytes.
    ///
    /// # Panics
    ///
    /// Panics if `self` has no private components, or if `to` is smaller
    /// than `self.size()`.
    pub fn private_decrypt(
        &self,
        from: &[u8],
        to: &mut [u8],
        padding: Padding,
    ) -> Result<usize, ErrorStack> {
        assert!(self.d().is_some(), "private components missing");
        assert!(from.len() <= i32::max_value() as usize);
        assert!(to.len() >= self.size() as usize);

        unsafe {
            let len = cvt_n(ffi::RSA_private_decrypt(
                from.len() as c_int,
                from.as_ptr(),
                to.as_mut_ptr(),
                self.as_ptr(),
                padding.0,
            ))?;
            Ok(len as usize)
        }
    }

    /// Encrypts data using the private key, returning the number of encrypted bytes.
    ///
    /// # Panics
    ///
    /// Panics if `self` has no private components, or if `to` is smaller
    /// than `self.size()`.
    pub fn private_encrypt(
        &self,
        from: &[u8],
        to: &mut [u8],
        padding: Padding,
    ) -> Result<usize, ErrorStack> {
        assert!(self.d().is_some(), "private components missing");
        assert!(from.len() <= i32::max_value() as usize);
        assert!(to.len() >= self.size() as usize);

        unsafe {
            let len = cvt_n(ffi::RSA_private_encrypt(
                from.len() as c_int,
                from.as_ptr(),
                to.as_mut_ptr(),
                self.as_ptr(),
                padding.0,
            ))?;
            Ok(len as usize)
        }
    }

    /// Decrypts data using the public key, returning the number of decrypted bytes.
    ///
    /// # Panics
    ///
    /// Panics if `to` is smaller than `self.size()`.
    pub fn public_decrypt(
        &self,
        from: &[u8],
        to: &mut [u8],
        padding: Padding,
    ) -> Result<usize, ErrorStack> {
        assert!(from.len() <= i32::max_value() as usize);
        assert!(to.len() >= self.size() as usize);

        unsafe {
            let len = cvt_n(ffi::RSA_public_decrypt(
                from.len() as c_int,
                from.as_ptr(),
                to.as_mut_ptr(),
                self.as_ptr(),
                padding.0,
            ))?;
            Ok(len as usize)
        }
    }

    /// Encrypts data using the public key, returning the number of encrypted bytes.
    ///
    /// # Panics
    ///
    /// Panics if `to` is smaller than `self.size()`.
    pub fn public_encrypt(
        &self,
        from: &[u8],
        to: &mut [u8],
        padding: Padding,
    ) -> Result<usize, ErrorStack> {
        assert!(from.len() <= i32::max_value() as usize);
        assert!(to.len() >= self.size() as usize);

        unsafe {
            let len = cvt_n(ffi::RSA_public_encrypt(
                from.len() as c_int,
                from.as_ptr(),
                to.as_mut_ptr(),
                self.as_ptr(),
                padding.0,
            ))?;
            Ok(len as usize)
        }
    }

    pub fn n(&self) -> Option<&BigNumRef> {
        unsafe {
            let n = compat::key(self.as_ptr())[0];
            if n.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(n as *mut _))
            }
        }
    }

    pub fn d(&self) -> Option<&BigNumRef> {
        unsafe {
            let d = compat::key(self.as_ptr())[2];
            if d.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(d as *mut _))
            }
        }
    }

    pub fn e(&self) -> Option<&BigNumRef> {
        unsafe {
            let e = compat::key(self.as_ptr())[1];
            if e.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(e as *mut _))
            }
        }
    }

    pub fn p(&self) -> Option<&BigNumRef> {
        unsafe {
            let p = compat::factors(self.as_ptr())[0];
            if p.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(p as *mut _))
            }
        }
    }

    pub fn q(&self) -> Option<&BigNumRef> {
        unsafe {
            let q = compat::factors(self.as_ptr())[1];
            if q.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(q as *mut _))
            }
        }
    }

    pub fn dp(&self) -> Option<&BigNumRef> {
        unsafe {
            let dp = compat::crt_params(self.as_ptr())[0];
            if dp.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(dp as *mut _))
            }
        }
    }

    pub fn dq(&self) -> Option<&BigNumRef> {
        unsafe {
            let dq = compat::crt_params(self.as_ptr())[1];
            if dq.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(dq as *mut _))
            }
        }
    }

    pub fn qi(&self) -> Option<&BigNumRef> {
        unsafe {
            let qi = compat::crt_params(self.as_ptr())[2];
            if qi.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(qi as *mut _))
            }
        }
    }
}

impl Rsa {
    /// only useful for associating the key material directly with the key, it's safer to use
    /// the supplied load and save methods for DER formatted keys.
    pub fn from_public_components(n: BigNum, e: BigNum) -> Result<Rsa, ErrorStack> {
        unsafe {
            let rsa = Rsa(cvt_p(ffi::RSA_new())?);
            cvt(compat::set_key(
                rsa.0,
                n.as_ptr(),
                e.as_ptr(),
                ptr::null_mut(),
            ))?;
            mem::forget((n, e));
            Ok(rsa)
        }
    }

    pub fn from_private_components(
        n: BigNum,
        e: BigNum,
        d: BigNum,
        p: BigNum,
        q: BigNum,
        dp: BigNum,
        dq: BigNum,
        qi: BigNum,
    ) -> Result<Rsa, ErrorStack> {
        unsafe {
            let rsa = Rsa(cvt_p(ffi::RSA_new())?);
            cvt(compat::set_key(rsa.0, n.as_ptr(), e.as_ptr(), d.as_ptr()))?;
            mem::forget((n, e, d));
            cvt(compat::set_factors(rsa.0, p.as_ptr(), q.as_ptr()))?;
            mem::forget((p, q));
            cvt(compat::set_crt_params(
                rsa.0,
                dp.as_ptr(),
                dq.as_ptr(),
                qi.as_ptr(),
            ))?;
            mem::forget((dp, dq, qi));
            Ok(rsa)
        }
    }

    /// Generates a public/private key pair with the specified size.
    ///
    /// The public exponent will be 65537.
    pub fn generate(bits: u32) -> Result<Rsa, ErrorStack> {
        ffi::init();
        unsafe {
            let rsa = Rsa(cvt_p(ffi::RSA_new())?);
            let e = BigNum::from_u32(ffi::RSA_F4 as u32)?;
            cvt(ffi::RSA_generate_key_ex(
                rsa.0,
                bits as c_int,
                e.as_ptr(),
                ptr::null_mut(),
            ))?;
            Ok(rsa)
        }
    }

    // FIXME these need to identify input formats
    private_key_from_pem!(Rsa, ffi::PEM_read_bio_RSAPrivateKey);
    private_key_from_der!(Rsa, ffi::d2i_RSAPrivateKey);
    public_key_from_pem!(Rsa, ffi::PEM_read_bio_RSA_PUBKEY);
    public_key_from_der!(Rsa, ffi::d2i_RSA_PUBKEY);

    from_der_inner!(
        /// Deserializes a public key from DER-encoded PKCS#1 data.
        public_key_from_der_pkcs1,
        Rsa,
        ffi::d2i_RSAPublicKey
    );
}

impl fmt::Debug for Rsa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rsa")
    }
}

#[cfg(ossl110)]
mod compat {
    use std::ptr;

    use ffi::{self, BIGNUM, RSA};
    use libc::c_int;

    pub unsafe fn key(r: *const RSA) -> [*const BIGNUM; 3] {
        let (mut n, mut e, mut d) = (ptr::null(), ptr::null(), ptr::null());
        ffi::RSA_get0_key(r, &mut n, &mut e, &mut d);
        [n, e, d]
    }

    pub unsafe fn factors(r: *const RSA) -> [*const BIGNUM; 2] {
        let (mut p, mut q) = (ptr::null(), ptr::null());
        ffi::RSA_get0_factors(r, &mut p, &mut q);
        [p, q]
    }

    pub unsafe fn crt_params(r: *const RSA) -> [*const BIGNUM; 3] {
        let (mut dp, mut dq, mut qi) = (ptr::null(), ptr::null(), ptr::null());
        ffi::RSA_get0_crt_params(r, &mut dp, &mut dq, &mut qi);
        [dp, dq, qi]
    }

    pub unsafe fn set_key(r: *mut RSA, n: *mut BIGNUM, e: *mut BIGNUM, d: *mut BIGNUM) -> c_int {
        ffi::RSA_set0_key(r, n, e, d)
    }

    pub unsafe fn set_factors(r: *mut RSA, p: *mut BIGNUM, q: *mut BIGNUM) -> c_int {
        ffi::RSA_set0_factors(r, p, q)
    }

    pub unsafe fn set_crt_params(
        r: *mut RSA,
        dmp1: *mut BIGNUM,
        dmq1: *mut BIGNUM,
        iqmp: *mut BIGNUM,
    ) -> c_int {
        ffi::RSA_set0_crt_params(r, dmp1, dmq1, iqmp)
    }
}

#[cfg(ossl10x)]
mod compat {
    use libc::c_int;
    use ffi::{BIGNUM, RSA};

    pub unsafe fn key(r: *const RSA) -> [*const BIGNUM; 3] {
        [(*r).n, (*r).e, (*r).d]
    }

    pub unsafe fn factors(r: *const RSA) -> [*const BIGNUM; 2] {
        [(*r).p, (*r).q]
    }

    pub unsafe fn crt_params(r: *const RSA) -> [*const BIGNUM; 3] {
        [(*r).dmp1, (*r).dmq1, (*r).iqmp]
    }

    pub unsafe fn set_key(r: *mut RSA, n: *mut BIGNUM, e: *mut BIGNUM, d: *mut BIGNUM) -> c_int {
        (*r).n = n;
        (*r).e = e;
        (*r).d = d;
        1 // TODO: is this right? should it be 0? what's success?
    }

    pub unsafe fn set_factors(r: *mut RSA, p: *mut BIGNUM, q: *mut BIGNUM) -> c_int {
        (*r).p = p;
        (*r).q = q;
        1 // TODO: is this right? should it be 0? what's success?
    }

    pub unsafe fn set_crt_params(
        r: *mut RSA,
        dmp1: *mut BIGNUM,
        dmq1: *mut BIGNUM,
        iqmp: *mut BIGNUM,
    ) -> c_int {
        (*r).dmp1 = dmp1;
        (*r).dmq1 = dmq1;
        (*r).iqmp = iqmp;
        1 // TODO: is this right? should it be 0? what's success?
    }
}

#[cfg(test)]
mod test {
    use symm::Cipher;

    use super::*;

    #[test]
    fn test_from_password() {
        let key = include_bytes!("../test/rsa-encrypted.pem");
        Rsa::private_key_from_pem_passphrase(key, b"mypass").unwrap();
    }

    #[test]
    fn test_from_password_callback() {
        let mut password_queried = false;
        let key = include_bytes!("../test/rsa-encrypted.pem");
        Rsa::private_key_from_pem_callback(key, |password| {
            password_queried = true;
            password[..6].copy_from_slice(b"mypass");
            Ok(6)
        }).unwrap();

        assert!(password_queried);
    }

    #[test]
    fn test_to_password() {
        let key = Rsa::generate(2048).unwrap();
        let pem = key.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), b"foobar")
            .unwrap();
        Rsa::private_key_from_pem_passphrase(&pem, b"foobar").unwrap();
        assert!(Rsa::private_key_from_pem_passphrase(&pem, b"fizzbuzz").is_err());
    }

    #[test]
    fn test_public_encrypt_private_decrypt_with_padding() {
        let key = include_bytes!("../test/rsa.pem.pub");
        let public_key = Rsa::public_key_from_pem(key).unwrap();

        let mut result = vec![0; public_key.size() as usize];
        let original_data = b"This is test";
        let len = public_key
            .public_encrypt(original_data, &mut result, Padding::PKCS1)
            .unwrap();
        assert_eq!(len, 256);

        let pkey = include_bytes!("../test/rsa.pem");
        let private_key = Rsa::private_key_from_pem(pkey).unwrap();
        let mut dec_result = vec![0; private_key.size() as usize];
        let len = private_key
            .private_decrypt(&result, &mut dec_result, Padding::PKCS1)
            .unwrap();

        assert_eq!(&dec_result[..len], original_data);
    }

    #[test]
    fn test_private_encrypt() {
        let k0 = super::Rsa::generate(512).unwrap();
        let k0pkey = k0.public_key_to_pem().unwrap();
        let k1 = super::Rsa::public_key_from_pem(&k0pkey).unwrap();

        let msg = vec![0xdeu8, 0xadu8, 0xd0u8, 0x0du8];

        let mut emesg = vec![0; k0.size() as usize];
        k0.private_encrypt(&msg, &mut emesg, Padding::PKCS1)
            .unwrap();
        let mut dmesg = vec![0; k1.size() as usize];
        let len = k1.public_decrypt(&emesg, &mut dmesg, Padding::PKCS1)
            .unwrap();
        assert_eq!(msg, &dmesg[..len]);
    }

    #[test]
    fn test_public_encrypt() {
        let k0 = super::Rsa::generate(512).unwrap();
        let k0pkey = k0.private_key_to_pem().unwrap();
        let k1 = super::Rsa::private_key_from_pem(&k0pkey).unwrap();

        let msg = vec![0xdeu8, 0xadu8, 0xd0u8, 0x0du8];

        let mut emesg = vec![0; k0.size() as usize];
        k0.public_encrypt(&msg, &mut emesg, Padding::PKCS1).unwrap();
        let mut dmesg = vec![0; k1.size() as usize];
        let len = k1.private_decrypt(&emesg, &mut dmesg, Padding::PKCS1)
            .unwrap();
        assert_eq!(msg, &dmesg[..len]);
    }
}
