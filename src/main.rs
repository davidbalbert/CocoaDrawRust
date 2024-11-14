use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSMenu, NSMenuItem, NSView, NSWindow, NSWindowStyleMask};
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::declare::ClassDecl;
use objc::{class, msg_send, sel, sel_impl};
use cocoa::base::{nil, YES, NO, id};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString, NSUInteger};
use bitflags::bitflags;

bitflags! {
    pub struct NSTrackingAreaOptions: NSUInteger {
        const MouseEnteredAndExited = 0x01;
        const MouseMoved = 0x02;
        const CursorUpdate = 0x04;
        const ActiveWhenFirstResponder = 0x10;
        const ActiveInKeyWindow = 0x20;
        const ActiveInActiveApp = 0x40;
        const ActiveAlways = 0x80;
        const AssumeInside = 0x100;
        const InVisibleRect = 0x200;
        const EnabledDuringMouseDrag = 0x400;
    }
}

#[allow(non_snake_case)]
trait NSTrackingArea: Sized {
    unsafe fn alloc(_: Self) -> id {
        msg_send![class!(NSTrackingArea), alloc]
    }

    unsafe fn initWithRect_options_owner_userInfo_(self, rect: NSRect, options: NSTrackingAreaOptions, owner: id, userInfo: id) -> id;
}

#[allow(non_snake_case)]
impl NSTrackingArea for id {
    unsafe fn initWithRect_options_owner_userInfo_(self, rect: NSRect, options: NSTrackingAreaOptions, owner: id, userInfo: id) -> id {
        msg_send![self, initWithRect:rect options:options owner:owner userInfo:userInfo]
    }
}

extern "C" fn my_view_init_with_frame(this: &mut Object, _cmd: Sel, frame: NSRect) -> id {
    unsafe {
        let this: id = msg_send![super(this, class!(NSView)), initWithFrame:frame];
        if this != nil {
            let tracking_area = NSTrackingArea::alloc(nil).initWithRect_options_owner_userInfo_(
                frame,
                // InVisibleRect makes the tracking area ignore self.rect and use the view's visibleRect instead. This means we don't have to
                // implement updateTrackingAreas to keep the tracking area's rect in sync with the view's frame.
                NSTrackingAreaOptions::MouseEnteredAndExited | NSTrackingAreaOptions::ActiveInKeyWindow | NSTrackingAreaOptions::InVisibleRect,
                this,
                nil,
            );
            let _: () = msg_send![this, addTrackingArea:tracking_area];
        }
        this
    }
}

#[allow(non_snake_case)]
trait MyView: Sized {
    unsafe fn alloc(_: Self) -> id {
        msg_send![my_view_class(), alloc]
    }
}

impl MyView for id {}

extern "C" fn my_view_draw_rect(this: &Object, _cmd: Sel, _rect: NSRect) {
    unsafe {
        let color: id = if msg_send![this, isMouseDown] {
            msg_send![class!(NSColor), redColor]
        } else if msg_send![this, isHovering] {
            msg_send![class!(NSColor), purpleColor]
        } else {
            msg_send![class!(NSColor), blueColor]
        };
        let _: id = msg_send![color, set];
        let path: id = msg_send![class!(NSBezierPath), bezierPathWithRect: _rect];
        let _: id = msg_send![path, fill];
    }
}

extern "C" fn my_view_mouse_down(this: &mut Object, _cmd: Sel, _event: id) {
    unsafe {
        let _: () = msg_send![this, setMouseIsDown: YES];
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn my_view_mouse_up(this: &mut Object, _cmd: Sel, _event: id) {
    unsafe {
        let _: () = msg_send![this, setMouseIsDown: NO];
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn my_view_mouse_entered(this: &mut Object, _cmd: Sel, _event: id) {
    unsafe {
        let _: () = msg_send![this, setIsHovering: YES];
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn my_view_mouse_exited(this: &mut Object, _cmd: Sel, _event: id) {
    unsafe {
        let _: () = msg_send![this, setIsHovering: NO];
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn my_view_is_mouse_down(this: &Object, _cmd: Sel) -> BOOL {
    unsafe {
        *this.get_ivar("isMouseDown")
    }
}

extern "C" fn my_view_set_is_mouse_down(this: &mut Object, _cmd: Sel, value: BOOL) {
    unsafe {
        this.set_ivar("isMouseDown", value);
    }
}

extern "C" fn my_view_is_hovering(this: &Object, _cmd: Sel) -> BOOL {
    unsafe {
        *this.get_ivar("isHovering")
    }
}

extern "C" fn my_view_set_is_hovering(this: &mut Object, _cmd: Sel, value: BOOL) {
    unsafe {
        this.set_ivar("isHovering", value);
    }
}

extern "C" fn my_view_class() -> *const Class {
    static mut MY_VIEW_CLASS: *const Class = 0 as *const Class;
    unsafe {
        if MY_VIEW_CLASS.is_null() {
            let superclass = class!(NSView);
            let mut decl = ClassDecl::new("MyView", superclass).unwrap();
            decl.add_ivar::<BOOL>("isMouseDown");
            decl.add_method(sel!(isMouseDown), my_view_is_mouse_down as extern "C" fn(&Object, Sel) -> BOOL);
            decl.add_method(sel!(setMouseIsDown:), my_view_set_is_mouse_down as extern "C" fn(&mut Object, Sel, BOOL));

            decl.add_ivar::<BOOL>("isHovering");
            decl.add_method(sel!(isHovering), my_view_is_hovering as extern "C" fn(&Object, Sel) -> BOOL);
            decl.add_method(sel!(setIsHovering:), my_view_set_is_hovering as extern "C" fn(&mut Object, Sel, BOOL));
            
            decl.add_method(sel!(initWithFrame:), my_view_init_with_frame as extern "C" fn(&mut Object, Sel, NSRect) -> id);
            decl.add_method(sel!(drawRect:), my_view_draw_rect as extern "C" fn(&Object, Sel, NSRect));
            decl.add_method(sel!(mouseDown:), my_view_mouse_down as extern "C" fn(&mut Object, Sel, id));
            decl.add_method(sel!(mouseUp:), my_view_mouse_up as extern "C" fn(&mut Object, Sel, id));
            decl.add_method(sel!(mouseEntered:), my_view_mouse_entered as extern "C" fn(&mut Object, Sel, id));
            decl.add_method(sel!(mouseExited:), my_view_mouse_exited as extern "C" fn(&mut Object, Sel, id));
            MY_VIEW_CLASS = decl.register();
        }
        MY_VIEW_CLASS
    }
}

fn main() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);


        let menubar = NSMenu::new(nil).autorelease();
        let app_menu_item = NSMenuItem::new(nil).autorelease();
        menubar.addItem_(app_menu_item);
        app.setMainMenu_(menubar);

        let app_menu = NSMenu::new(nil).autorelease();
        let quit_title = NSString::alloc(nil).init_str("Quit CocoaDrawRust");
        let quit_action = sel!(terminate:);
        let quit_key = NSString::alloc(nil).init_str("q");
        let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(quit_title, quit_action, quit_key).autorelease();
        app_menu.addItem_(quit_item);
        app_menu_item.setSubmenu_(app_menu);

        let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 600.0)),
            NSWindowStyleMask::NSTitledWindowMask | NSWindowStyleMask::NSClosableWindowMask | NSWindowStyleMask::NSResizableWindowMask,
            NSBackingStoreType::NSBackingStoreBuffered,
            NO,
        ).autorelease();

        window.setTitle_(NSString::alloc(nil).init_str("Hello, World!"));

        // Frame size doesn't matter. NSWindow updates the size of its content view to match its size.
        let v = MyView::alloc(nil).initWithFrame_(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)));
        window.setContentView_(v);

        window.center();
        window.makeKeyAndOrderFront_(nil);

        app.run();
    }
}
