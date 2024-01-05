import Foundation
import SharedTypes

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    init() {
        self.view = try! .bincodeDeserialize(input: [UInt8](AnimalHunt.view()))
    }

    func update(_ event: Event) {
        print("Update", event)
        
        let effects = [UInt8](processEvent(Data(try! event.bincodeSerialize())))

        let requests: [Request] = try! .bincodeDeserialize(input: effects)
        for request in requests {
            processEffect(request)
        }
    }

    func processEffect(_ request: Request) {
        switch request.effect {
        case .render:
            print("Render")
            view = try! .bincodeDeserialize(input: [UInt8](AnimalHunt.view()))
        case let .tagReader(req):
            print("Tag reader request", req)
        }
    }   
}
