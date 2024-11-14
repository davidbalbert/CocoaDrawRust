use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSMenu, NSMenuItem, NSView, NSWindow, NSWindowStyleMask};
use objc::runtime::{Class, Object, Sel, BOOL};
use objc::declare::ClassDecl;
use objc::{class, msg_send, sel, sel_impl};
use cocoa::base::{nil, YES, NO, id};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};

extern "C" fn my_view_draw_rect(this: &Object, _cmd: Sel, _rect: NSRect) {
    let _ = this;
    unsafe {
        let is_mouse_down: BOOL = msg_send![this, isMouseDown];
        let color: id = if is_mouse_down == YES {
            msg_send![class!(NSColor), redColor]
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

extern "C" fn my_view_mouse_is_down(this: &Object, _cmd: Sel) -> BOOL {
    unsafe {
        *this.get_ivar("mouseIsDown")
    }
}

extern "C" fn my_view_set_mouse_is_down(this: &mut Object, _cmd: Sel, value: BOOL) {
    unsafe {
        this.set_ivar("mouseIsDown", value);
    }
}

extern "C" fn my_view_class() -> *const Class {
    static mut MY_VIEW_CLASS: *const Class = 0 as *const Class;
    unsafe {
        if MY_VIEW_CLASS.is_null() {
            let superclass = class!(NSView);
            let mut decl = ClassDecl::new("MyView", superclass).unwrap();
            decl.add_ivar::<BOOL>("mouseIsDown");
            decl.add_method(sel!(isMouseDown), my_view_mouse_is_down as extern "C" fn(&Object, Sel) -> BOOL);
            decl.add_method(sel!(setMouseIsDown:), my_view_set_mouse_is_down as extern "C" fn(&mut Object, Sel, BOOL));
            
            decl.add_method(sel!(drawRect:), my_view_draw_rect as extern "C" fn(&Object, Sel, NSRect));
            decl.add_method(sel!(mouseDown:), my_view_mouse_down as extern "C" fn(&mut Object, Sel, id));
            decl.add_method(sel!(mouseUp:), my_view_mouse_up as extern "C" fn(&mut Object, Sel, id));
            MY_VIEW_CLASS = decl.register();
        }
        MY_VIEW_CLASS
    }
}

trait MyView: Sized {
    unsafe fn alloc(_: Self) -> id {
        msg_send![my_view_class(), alloc]
    }
}

impl MyView for id {}

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
        let quit_title = NSString::alloc(nil).init_str("Quit");
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

        let v = MyView::alloc(nil).initWithFrame_(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 600.0)));
        window.setContentView_(v);

        window.center();
        window.makeKeyAndOrderFront_(nil);

        app.run();
    }
}
