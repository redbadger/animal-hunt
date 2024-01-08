import SharedTypes
import SwiftUI

struct ConfigureView: View {
    @Environment(\.update) var update
    @Environment(\.presentationMode) var presentation

    var animals: [[String]]

    var error: String?

    @State private var selectedAnimal: String? = nil

    var body: some View {
        ZStack(alignment: Alignment(horizontal: .center, vertical: .bottom)) {
            VStack(alignment: .leading) {
                List(animals, id: \.[0], selection: $selectedAnimal) { animal in
                    HStack {
                        Text(animal[1])

                        Text(animal[0].capitalized)
                    }
                }
                .onChange(of: selectedAnimal) { animal in
                    if let animal {
                        update(.writeTag(animal))
                    }
                }
            }

            if let error {
                Text(error)
                    .foregroundColor(.white)
                    .padding(.horizontal, 25)
                    .padding(.vertical, 10)
                    .background(.red)
                    .cornerRadius(20)
                    .padding(10)
                    .transition(AnyTransition.push(from: .leading))
                    .animation(.easeInOut, value: error)
            }
        }
            .animation(.bouncy(duration: 0.3, extraBounce: 0.2), value: error)
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

        ConfigureView(animals: [
            ["dog", "🐕"],
            ["cat", "🐈‍⬛"],
            ["badger", "🦡"],
        ], error: "Oh no! Problems!")
        .environment(\.update, { e in print("Event triggered", e) })
    }
}
