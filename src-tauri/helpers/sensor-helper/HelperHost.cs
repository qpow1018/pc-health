using System.Text.Json;

namespace PcHealth.SensorHelper;

public interface ICpuSensorSampler
{
    IReadOnlyList<HardwareRecord> Sample(ICollection<string> errors);
}

public sealed class HelperHost(ICpuSensorSampler sampler, int helperPid)
{
    private long sampleIndex;

    public void Run(TextReader input, TextWriter output, TextWriter error)
    {
        string? line;
        while ((line = input.ReadLine()) is not null)
        {
            if (!TryReadRequest(line, out var request, out var protocolError))
            {
                WriteResponse(
                    output,
                    new HelperResponse(
                        null,
                        false,
                        helperPid,
                        sampleIndex,
                        "protocol",
                        [],
                        [protocolError]));
                continue;
            }

            if (request.Command == "shutdown")
            {
                WriteResponse(
                    output,
                    new HelperResponse(
                        request.Id,
                        true,
                        helperPid,
                        sampleIndex,
                        "shutdown",
                        [],
                        []));
                return;
            }

            if (request.Command != "sample" || request.Id is null)
            {
                WriteResponse(
                    output,
                    new HelperResponse(
                        request.Id,
                        false,
                        helperPid,
                        sampleIndex,
                        request.Command ?? "protocol",
                        [],
                        ["Unknown command or missing request id."]));
                continue;
            }

            sampleIndex += 1;
            var errors = new List<string>();
            IReadOnlyList<HardwareRecord> hardware;
            try
            {
                hardware = sampler.Sample(errors);
            }
            catch (Exception exception)
            {
                errors.Add(exception.Message);
                hardware = [];
            }

            WriteResponse(
                output,
                new HelperResponse(
                    request.Id,
                    hardware.Count > 0 || errors.Count == 0,
                    helperPid,
                    sampleIndex,
                    "sample",
                    hardware,
                    errors));
        }

        error.WriteLine("Sensor helper stdin reached EOF.");
        error.Flush();
    }

    private static bool TryReadRequest(
        string line,
        out HelperRequest request,
        out string protocolError)
    {
        try
        {
            request = JsonSerializer.Deserialize<HelperRequest>(line, ProtocolJson.Options)
                ?? new HelperRequest(null, null);
            protocolError = string.Empty;
            return true;
        }
        catch (JsonException exception)
        {
            request = new HelperRequest(null, null);
            protocolError = $"Invalid JSON request: {exception.Message}";
            return false;
        }
    }

    private static void WriteResponse(TextWriter output, HelperResponse response)
    {
        output.WriteLine(JsonSerializer.Serialize(response, ProtocolJson.Options));
        output.Flush();
    }
}
