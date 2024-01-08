import SharedTypes
import SwiftUI

struct PracticeView: View {
    @Environment(\.update) var update

    var animalEmoji: String
    var error: String?

    var body: some View {
        ZStack(alignment: Alignment(horizontal: .center, vertical: .bottom)) {
            VStack {
                Text(animalEmoji)
                    .font(.system(size: 150))
                    .aspectRatio(1.0, contentMode: .fit)
                    .frame(width: 200, height: 200)
                    .foregroundColor(.gray)
                    .padding(30)
                    .background(.white)
                    .cornerRadius(20)

                Spacer()
                    .frame(maxHeight: 40)

                Button {
                    update(.scan)
                } label: {
                    Text("Scan")
                        .font(.title)
                }
                .foregroundColor(.white)
                .padding(.horizontal, 25)
                .padding(.vertical, 10)
                .background(.blue)
                .cornerRadius(20)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color(hue: 0, saturation: 0, brightness: 0.95))

            if let err = error {
                Button {
                    update(.dismissError)
                } label: {
                    Text(err)
                        .foregroundColor(.white)
                        .padding(.horizontal, 25)
                        .padding(.vertical, 10)
                        .background(.red)
                        .cornerRadius(20)
                        .padding(10)
                }
                .transition(
                   AnyTransition.push(from: .bottom)
                )
            }
        }
        .animation(.bouncy(duration: 0.6, extraBounce: 0.2), value: error)
        .navigationBarTitleDisplayMode(.inline)
        .navigationTitle("Practice")
        .toolbar() {
            ToolbarItem(placement: .primaryAction) {
                Button("Configure Tag") {
                    print("Configure pressed")
                    update(.setMode(.configure))
                }
            }
        }
    }
}

struct PracticeView_Previews: PreviewProvider {
    static var previews: some View {
        PracticeView(animalEmoji: "🦩")

        PracticeView(animalEmoji: "?", error: "Something went wrong!")
    }
}
