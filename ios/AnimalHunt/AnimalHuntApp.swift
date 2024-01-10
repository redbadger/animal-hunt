import SwiftUI

@main
struct AnimalHuntApp: App {
    private let core: Core

    init() {
        core = Core()
    }

    var body: some Scene {
        WindowGroup {
            RootView(core: core).onOpenURL(perform: { url in
                let url = url.absoluteString

                core.update(.scannedUrl(.url(url)))
            }).onContinueUserActivity(NSUserActivityTypeBrowsingWeb, perform: { userActivity in
                guard let url = userActivity.webpageURL?.absoluteString else {
                    return
                }
                
                core.update(.scannedUrl(.url(url)))
            })
        }
    }
}
