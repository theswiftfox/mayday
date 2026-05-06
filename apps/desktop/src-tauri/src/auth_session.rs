//! macOS ASWebAuthenticationSession wrapper.
//!
//! Uses the system authentication session which benefits from the Enterprise SSO
//! extension (Microsoft Company Portal) for device compliance/conditional access.
//!
//! This intercepts the urn:ietf:wg:oauth:2.0:oob redirect and returns the full
//! callback URL containing the authorization code.

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, AnyProtocol, Bool};
use objc2::sel;
use objc2::msg_send;
use objc2_foundation::{NSString, NSURL};

use std::sync::{Arc, Condvar, Mutex};

/// Result holder shared between the completion block and the caller
struct AuthResult {
    value: Mutex<Option<Result<String, String>>>,
    condvar: Condvar,
}

/// Register a minimal class that implements ASWebAuthenticationPresentationContextProviding.
/// Returns the app's key window as the presentation anchor.
unsafe fn get_presentation_context_provider() -> Retained<AnyObject> {
    use objc2::runtime::ClassBuilder;
    use std::sync::Once;

    static REGISTER: Once = Once::new();

    REGISTER.call_once(|| {
        let superclass = AnyClass::get(c"NSObject").unwrap();
        let mut builder = ClassBuilder::new(c"MydayAuthPresentationContext", superclass).unwrap();

        // Add protocol conformance
        let protocol = AnyProtocol::get(c"ASWebAuthenticationPresentationContextProviding").unwrap();
        builder.add_protocol(protocol);

        // Add the required method: presentationAnchorForWebAuthenticationSession:
        unsafe extern "C" fn presentation_anchor(
            _this: *mut AnyObject,
            _sel: objc2::runtime::Sel,
            _session: *mut AnyObject,
        ) -> *mut AnyObject {
            // Get NSApp.keyWindow
            let app: *mut AnyObject = msg_send![AnyClass::get(c"NSApplication").unwrap(), sharedApplication];
            let window: *mut AnyObject = msg_send![app, keyWindow];
            window
        }

        let sel = sel!(presentationAnchorForWebAuthenticationSession:);
        builder.add_method(
            sel,
            presentation_anchor as unsafe extern "C" fn(*mut AnyObject, objc2::runtime::Sel, *mut AnyObject) -> *mut AnyObject,
        );

        builder.register();
    });

    let cls = AnyClass::get(c"MydayAuthPresentationContext").unwrap();
    msg_send![cls, new]
}

/// Run an ASWebAuthenticationSession and block until it completes.
/// Sends the result (callback URL or error) through the oneshot channel.
pub fn run_auth_session(url_str: &str, tx: tokio::sync::oneshot::Sender<Result<String, String>>) {
    let result = Arc::new(AuthResult {
        value: Mutex::new(None),
        condvar: Condvar::new(),
    });

    let url_string = url_str.to_string();
    let result_clone = result.clone();

    // ASWebAuthenticationSession must be started from the main thread
    dispatch::Queue::main().exec_async(move || {
        unsafe {
            let ns_url_str = NSString::from_str(&url_string);
            let ns_url: Option<Retained<NSURL>> = msg_send![
                AnyClass::get(c"NSURL").unwrap(),
                URLWithString: &*ns_url_str
            ];
            let ns_url = match ns_url {
                Some(u) => u,
                None => {
                    let mut lock = result_clone.value.lock().unwrap();
                    *lock = Some(Err("Invalid URL".to_string()));
                    result_clone.condvar.notify_one();
                    return;
                }
            };

            // Create callback scheme for "urn" to intercept urn:ietf:wg:oauth:2.0:oob
            let scheme = NSString::from_str("urn");

            // ASWebAuthenticationSessionCallback.callbackWithCustomScheme:
            let callback_cls = AnyClass::get(c"ASWebAuthenticationSessionCallback").unwrap();
            let callback: Retained<AnyObject> =
                msg_send![callback_cls, callbackWithCustomScheme: &*scheme];

            // Create the completion handler block
            let result_for_block = result_clone.clone();
            let completion_block = block2::StackBlock::new(
                move |callback_url: *mut AnyObject, error: *mut AnyObject| {
                    let result_val = if !callback_url.is_null() {
                        let abs_str: Retained<NSString> =
                            msg_send![callback_url, absoluteString];
                        Ok(abs_str.to_string())
                    } else if !error.is_null() {
                        let desc: Retained<NSString> =
                            msg_send![error, localizedDescription];
                        Err(desc.to_string())
                    } else {
                        Err("Auth session cancelled".to_string())
                    };

                    let mut lock = result_for_block.value.lock().unwrap();
                    *lock = Some(result_val);
                    result_for_block.condvar.notify_one();
                },
            );
            let completion_block = completion_block.copy();

            // Create ASWebAuthenticationSession
            let session_cls = AnyClass::get(c"ASWebAuthenticationSession").unwrap();
            let session: Retained<AnyObject> = msg_send![
                msg_send![session_cls, alloc],
                initWithURL: &*ns_url,
                callback: &*callback,
                completionHandler: &*completion_block
            ];

            // Set presentation context provider (REQUIRED on macOS for the session to show)
            let context = get_presentation_context_provider();
            let _: () = msg_send![&session, setPresentationContextProvider: &*context];

            // Start the session
            let started: Bool = msg_send![&session, start];
            if !started.as_bool() {
                let mut lock = result_clone.value.lock().unwrap();
                *lock = Some(Err("Failed to start ASWebAuthenticationSession".to_string()));
                result_clone.condvar.notify_one();
                return;
            }

            // Keep session and context alive until completion
            std::mem::forget(session);
            std::mem::forget(context);
        }
    });

    // Wait for the result (the completion handler will signal us)
    let lock = result.value.lock().unwrap();
    let lock = result
        .condvar
        .wait_while(lock, |val| val.is_none())
        .unwrap();

    if let Some(val) = lock.clone() {
        let _ = tx.send(val);
    }
}
