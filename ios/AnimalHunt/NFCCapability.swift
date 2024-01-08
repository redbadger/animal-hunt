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
                return mapError(e)
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
                return mapError(e)
            }
        }
    }

    private static func mapError(_ error: NFCCapabilityError) -> TagReaderOutput {
        if case let .nfcSessionError(e) = error {
            if let nfcError = e as? NFCReaderError {
                if nfcError.code == .readerSessionInvalidationErrorUserCanceled ||
                    nfcError.code == .readerSessionInvalidationErrorSessionTimeout {

                    return .cancelled
                }
            }
        }

        return .error(String.init(format: "NFC read failed %@", error.localizedDescription))
    }
}

class NFCReadTransaction: NSObject {
    private var readerSession: NFCNDEFReaderSession? = nil

    private var continuation: CheckedContinuation<Result<String, NFCCapabilityError> , Never>?

    func commit() async -> Result<String, NFCCapabilityError> {
        self.readerSession = NFCNDEFReaderSession(delegate: self, queue: nil, invalidateAfterFirstRead: false)
        readerSession?.alertMessage = "Move the top of your iPhone close to the tag"

        return await withCheckedContinuation { continuation in
            self.continuation = continuation

            readerSession?.begin()
        }
    }

    func complete(url: String) {
        readerSession?.invalidate()

        continuation?.resume(returning: .success(url))
    }

    func fail(_ e: NFCCapabilityError) {
        readerSession?.invalidate()

        continuation?.resume(returning: .failure(e))
    }

}

extension NFCReadTransaction: NFCNDEFReaderSessionDelegate {
    func readerSessionDidBecomeActive(_ session: NFCNDEFReaderSession) {
    }

    func readerSession(_ session: NFCNDEFReaderSession, didInvalidateWithError error: Error) {
        fail(.nfcSessionError(error))
    }

    func readerSession(_ session: NFCNDEFReaderSession, didDetectNDEFs messages: [NFCNDEFMessage]) {

        if messages.count > 1 {
            return fail(.tooManyTagsDetected)
        }

        for record in messages[0].records {
            if let url = record.wellKnownTypeURIPayload() {
                session.alertMessage = "Tag read!"

                return self.complete(url: url.absoluteString)
            }
        }

        self.fail(.unsupportedTagType)
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

        self.readerSession = NFCNDEFReaderSession(delegate: self, queue: nil, invalidateAfterFirstRead: false)
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
