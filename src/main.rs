use std::cell::{OnceCell, RefCell};

use objc2::runtime::{ProtocolObject};
use objc2::{declare_class, msg_send_id, sel};
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType, NSBezierPath, NSColor, NSMenu, NSMenuItem, NSTrackingArea, NSTrackingAreaOptions, NSView, NSWindow, NSWindowStyleMask};
use objc2::mutability;
use objc2::ClassType;
use objc2::DeclaredClass;
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};

struct MyViewIvars {
    is_mouse_down: bool,
    is_hovering: bool,
}

declare_class!(
    struct MyView;

    unsafe impl ClassType for MyView {
        type Super = NSView;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "MyView";
    }

    impl DeclaredClass for MyView {
        type Ivars = RefCell<MyViewIvars>;
    }

    unsafe impl MyView {
        #[method(drawRect:)]
        fn draw_rect(&self, rect: NSRect) {
            unsafe {
                if self.ivars().borrow().is_mouse_down {
                    NSColor::redColor().set();
                } else if self.ivars().borrow().is_hovering {
                    NSColor::purpleColor().set();
                } else {
                    NSColor::blueColor().set();
                }
                NSBezierPath::fillRect(rect);
            }
        }

        #[method(mouseDown:)]
        fn mouse_down(&self, _event: *mut NSObject) {
            self.ivars().borrow_mut().is_mouse_down = true;
            unsafe {
                self.setNeedsDisplay(true);
            }
        }

        #[method(mouseUp:)]
        fn mouse_up(&self, _event: *mut NSObject) {
            self.ivars().borrow_mut().is_mouse_down = false;
            unsafe {
                self.setNeedsDisplay(true);
            }
        }

        #[method(mouseEntered:)]
        fn mouse_entered(&self, _event: *mut NSObject) {
            self.ivars().borrow_mut().is_hovering = true;
            unsafe {
                self.setNeedsDisplay(true);
            }
        }

        #[method(mouseExited:)]
        fn mouse_exited(&self, _event: *mut NSObject) {
            self.ivars().borrow_mut().is_hovering = false;
            unsafe {
                self.setNeedsDisplay(true);
            }
        }
    }
);

impl MyView {
    fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(RefCell::new(MyViewIvars {
            is_mouse_down: false,
            is_hovering: false,
        }));

        let this: Retained<MyView> = unsafe { msg_send_id![super(this), initWithFrame:frame] };

        let tracking_area = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                mtm.alloc(),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
                // InVisibleRect makes the tracking area ignore self.rect and use the view's visibleRect instead. This means we don't have to
                // implement updateTrackingAreas to keep the tracking area's rect in sync with the view's frame.
                NSTrackingAreaOptions::NSTrackingMouseEnteredAndExited |
                    NSTrackingAreaOptions::NSTrackingActiveAlways | 
                    NSTrackingAreaOptions::NSTrackingInVisibleRect,
                Some(&*this),
                None,
            )
        };

        unsafe { this.addTrackingArea(&tracking_area); }

        return this;
    }
}

struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
}

declare_class!(
    struct AppDelegate;

    unsafe impl ClassType for AppDelegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "AppDelegate";
    }

    impl DeclaredClass for AppDelegate {
        type Ivars = AppDelegateIvars;
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[method(applicationDidFinishLaunching:)]
        fn application_did_finish_launching(&self, _notification: *mut NSObject) {
            let mtm = MainThreadMarker::from(self);

            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Regular);    
            unsafe { app.activate(); }

            let main_menu = NSMenu::new(mtm);
            let app_menu_item = NSMenuItem::new(mtm);
            main_menu.addItem(&app_menu_item);

            let app_menu = NSMenu::new(mtm);
            let quit_item = NSMenuItem::new(mtm);
            unsafe {
                quit_item.setTitle(&NSString::from_str("Quit CocoaDrawRust"));
                quit_item.setAction(Some(sel!(terminate:)));
                quit_item.setKeyEquivalent(&NSString::from_str("q"));
            }
            app_menu.addItem(&quit_item);
            app_menu_item.setSubmenu(Some(&app_menu));
            app.setMainMenu(Some(&main_menu));

            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    mtm.alloc(),
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 600.0)),
                    NSWindowStyleMask::Closable | NSWindowStyleMask::Resizable | NSWindowStyleMask::Titled,
                    NSBackingStoreType::NSBackingStoreBuffered,
                    false,
                )
            };
            window.setTitle(&NSString::from_str("CocoaDrawRust"));

            // Frame size doesn't matter. NSWindow updates the size of its content view to match its size.
            let view = MyView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)));
            window.setContentView(Some(&view));

            window.center();
            window.makeKeyAndOrderFront(None);

            self.ivars().window.set(window).unwrap();
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(AppDelegateIvars {
            window: OnceCell::new(),
        });
        unsafe { msg_send_id![super(this), init] }
    }
}

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    let delegate = AppDelegate::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    unsafe { app.run(); }
}
