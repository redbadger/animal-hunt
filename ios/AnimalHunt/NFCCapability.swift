import Foundation
import CoreNFC
import SharedTypes

enum NFCCapabilityError: Error {
    case malformedURL
    case unsupportedTagType
    case tooManyTagsDetected
    case nfcSessionError(Error)
    case tagConnectionError(Error)
    case tagStatusError(Error)
    case tagReadError(Error)
    case tagWriteError(Error)
    case tagReadOnly
    case tagCapacityNotEnough
    case inconsistentState // should never happen
}

class NFCCapability: NSObject {

    static func process(_ operation: TagReaderOperation) async -> TagReaderOutput {
        switch operation {
        case let .writeUrl(url):
            guard NFCNDEFReaderSession.readingAvailable else {
                return .error("NFC reading not available")
            }

            let transaction = NFCWriteTransaction(urlString: url)

            switch await transaction.commit() {
            case .success():
                return .written
            case let .failure(e):
                return .error("NFC write failed") // TODO pass error message
            }
        case .readUrl:
            guard NFCNDEFReaderSession.readingAvailable else {
                return .error("NFC reading not available")
            }

            let transaction = NFCReadTransaction()

            switch await transaction.commit() {
            case let .success(url):
                return .url(url)
            case let .failure(e):
                return .error("NFC read failed") // TODO pass error message
            }
        }
    }
}

class NFCReadTransaction: NSObject {
    private var readerSession: NFCNDEFReaderSession? = nil

    private var continuation: CheckedContinuation<Result<String, NFCCapabilityError> , Never>?

    func commit() async -> Result<String, NFCCapabilityError> {
        self.readerSession = NFCNDEFReaderSession(delegate: self, queue: DispatchQueue.main, invalidateAfterFirstRead: true)
        readerSession?.alertMessage = "Move the top of your iPhone close to the tag"

        return await withCheckedContinuation { continuation in
            self.continuation = continuation

            readerSession?.begin()
            print("Session started")
        }
    }

    func complete(url: String) {
        readerSession?
        print("Session complete (success)")

        continuation?.resume(returning: .success(url))
    }

    func fail(_ e: NFCCapabilityError) {
        readerSession?.invalidate()
        print("Session complete (failure)")

        continuation?.resume(returning: .failure(e))
    }

}

extension NFCReadTransaction: NFCNDEFReaderSessionDelegate {
    func readerSessionDidBecomeActive(_ session: NFCNDEFReaderSession) {
        print("Session active")
    }

    func readerSession(_ session: NFCNDEFReaderSession, didInvalidateWithError error: Error) {
        if let nfcError = error as? NFCReaderError {
            if nfcError.code == .readerSessionInvalidationErrorUserCanceled ||
                nfcError.code == .readerSessionInvalidationErrorSessionTimeout {
                
                return complete(url: "https://animal-hunt.red-badger.com/animal/unknown") // FIXME error handling
            }
        }

        fail(.nfcSessionError(error))
    }

    func readerSession(_ session: NFCNDEFReaderSession, didDetectNDEFs messages: [NFCNDEFMessage]) {
        print("NDEF Message!")
    }

    func readerSession(_ session: NFCNDEFReaderSession, didDetect tags: [NFCNDEFTag]) {
        if tags.count > 1 {
            fail(.tooManyTagsDetected)
        }

        let tag = tags[0]

        session.connect(to: tag) { error in
            if let error {
                return self.fail(.tagConnectionError(error))
            }

            tag.readNDEF { message, error in
                if let error {
                    return self.fail(.tagReadError(error))
                }

                for record in message?.records ?? [] {
                    if let url = record.wellKnownTypeURIPayload() {
                        session.alertMessage = "Tag read!"

                        return self.complete(url: url.absoluteString)
                    }
                }

                self.fail(.unsupportedTagType)
            }
        }
    }
}

class NFCWriteTransaction: NSObject {

    private var urlString: String

    private var readerSession: NFCNDEFReaderSession? = nil
    private var ndefMessage: NFCNDEFMessage? = nil

    private var continuation: CheckedContinuation<Result<(), NFCCapabilityError> , Never>?

    init(urlString: String) {
        self.urlString = urlString
        self.ndefMessage = nil
        self.continuation = nil

        super.init()
    }

    func commit() async -> Result<(), NFCCapabilityError> {
        guard let payload = NFCNDEFPayload.wellKnownTypeURIPayload(string: urlString) else {
            return .failure(.malformedURL)
        }

        self.readerSession = NFCNDEFReaderSession(delegate: self, queue: DispatchQueue.main, invalidateAfterFirstRead: false)
        readerSession?.alertMessage = "Move the top of your iPhone close to the tag"

        ndefMessage = NFCNDEFMessage(records: [payload])

        return await withCheckedContinuation { continuation in
            self.continuation = continuation

            readerSession?.begin()
       }
    }

    func complete() {
        readerSession?.invalidate()
        continuation?.resume(returning: .success(()))
    }

    func fail(_ e: NFCCapabilityError) {
        readerSession?.invalidate()
        continuation?.resume(returning: .failure(e))
    }

    func writeTag(_ nfcTag: NFCNDEFTag, session: NFCNDEFReaderSession) {
        guard let ndefMessage else {
            return fail(.inconsistentState)
        }

        session.connect(to: nfcTag) { error  in
            if let error {
                return self.fail(.tagConnectionError(error))
            }

            print("Connection to tag sucessful")

            nfcTag.queryNDEFStatus { status, capacity, error in
                if let error {
                    return self.fail(.tagStatusError(error))
                }

                if status == .readOnly {
                    return self.fail(.tagReadOnly)
                }

                if ndefMessage.length > capacity {
                    return self.fail(.tagCapacityNotEnough)
                }

                nfcTag.writeNDEF(ndefMessage) { error in
                    if let error {
                        return self.fail(.tagWriteError(error))
                    }

                    session.alertMessage = "Tag written!"

                    self.complete()
                }
            }
        }
    }
}

extension NFCWriteTransaction: NFCNDEFReaderSessionDelegate {

    func readerSession(_ session: NFCNDEFReaderSession, didInvalidateWithError error: Error) {
        if let nfcError = error as? NFCReaderError {
            if nfcError.code == .readerSessionInvalidationErrorUserCanceled ||
                nfcError.code == .readerSessionInvalidationErrorSessionTimeout {
                return complete()
            }
        }

        fail(.nfcSessionError(error))
    }

    func readerSession(_ session: NFCNDEFReaderSession, didDetectNDEFs messages: [NFCNDEFMessage]) {}

    func readerSession(_ session: NFCNDEFReaderSession, didDetect tags: [NFCNDEFTag]) {
        if tags.count > 1 {
            fail(.tooManyTagsDetected)
        }

        writeTag(tags[0], session: session)
    }
}
