import SharedTypes
import SwiftUI

struct RootView: View {
    @Environment(\.presentationMode) var presentation
    @ObservedObject var core: Core

    private var configuring: Binding<Bool> {
        Binding (
            get: {
                switch self.core.view {
                case .practice:
                    return false
                case .configure:
                    return true
                }
            },
            set: { c in
                if !c, case .configure = core.view {
                    self.core.update(.setMode(.practice))
                }
            }
        )
    }
    private var animalEmoji: String {
        if case let .practice(animalEmoji) = core.view {
            return animalEmoji
        } else {
            return "?"
        }
    }
    private var knownAnimals: [[String]] {
        guard case let .configure(animals) = core.view else {
            return [];
        }

        return animals
    }

    var body: some View {


        NavigationStack {
            PracticeView(animalEmoji: animalEmoji)
                .sheet(isPresented: configuring) {

                    AnyView(NavigationStack {
                        ConfigureView(animals: knownAnimals)
                    })
                }
        }
        .environment(\.update, { e in core.update(e)})
    }
}

private struct UpdateKey: EnvironmentKey {
    static let defaultValue: (Event) -> Void = { _ in }
}

extension EnvironmentValues {
  var update: (Event) -> Void {
    get { self[UpdateKey.self] }
    set { self[UpdateKey.self] = newValue }
  }
}

struct RootView_Previews: PreviewProvider {
    static var previews: some View {
        RootView(core: Core())
    }
}
