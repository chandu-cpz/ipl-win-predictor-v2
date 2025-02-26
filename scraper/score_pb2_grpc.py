# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc

import score_pb2 as score__pb2


class ScoreStreamStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.StreamScore = channel.unary_stream(
                '/score.ScoreStream/StreamScore',
                request_serializer=score__pb2.StreamScoreRequest.SerializeToString,
                response_deserializer=score__pb2.StreamScoreResponse.FromString,
                )


class ScoreStreamServicer(object):
    """Missing associated documentation comment in .proto file."""

    def StreamScore(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_ScoreStreamServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'StreamScore': grpc.unary_stream_rpc_method_handler(
                    servicer.StreamScore,
                    request_deserializer=score__pb2.StreamScoreRequest.FromString,
                    response_serializer=score__pb2.StreamScoreResponse.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'score.ScoreStream', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class ScoreStream(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def StreamScore(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(request, target, '/score.ScoreStream/StreamScore',
            score__pb2.StreamScoreRequest.SerializeToString,
            score__pb2.StreamScoreResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)
