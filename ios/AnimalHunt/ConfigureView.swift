import SharedTypes
import SwiftUI

struct ConfigureView: View {
    @Environment(\.update) var update
    @Environment(\.presentationMode) var presentation

    var animals: [[String]]

    @State private var selectedAnimal: String? = nil

    var body: some View {
        VStack(alignment: .leading) {
            List(animals, id: \.[0], selection: $selectedAnimal) { animal in
                HStack {
                    Text(animal[1])

                    Text(animal[0].capitalized)
                }
            }
            .onChange(of: selectedAnimal) { animal in
                if animal != nil {
                    // TODO update(.writeTag(animal))
                    update(.tagWritten(.written))
                }
            }
        }
            .navigationBarTitleDisplayMode(.inline)
            .navigationTitle("Configure a tag")
            .toolbar {
                ToolbarItem(placement: .cancellationAction){
                    Button("Close") {
                        print("Close pressed")
                        presentation.wrappedValue.dismiss()
                    }
                }
            }
    }
}

struct ConfigureView_Previews: PreviewProvider {
    static var previews: some View {
        ConfigureView(animals: [
            ["dog", "🐕"],
            ["cat", "🐈‍⬛"],
            ["badger", "🦡"],
        ])
        .environment(\.update, { e in print("Event triggered", e) })
    }
}
