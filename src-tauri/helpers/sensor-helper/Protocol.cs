using System.Text.Json;
using LibreHardwareMonitor.Hardware;

namespace PcHealth.SensorHelper;

public sealed record HelperRequest(long? Id, string? Command);

public sealed record SensorRecord(
    string Name,
    string SensorType,
    string Identifier,
    string ParentIdentifier,
    string? Unit,
    double? Value);

public sealed record HardwareRecord(
    string Name,
    string HardwareType,
    string Identifier,
    string? ParentIdentifier,
    string? UpdateError,
    IReadOnlyList<SensorRecord> Sensors);

public sealed record HelperResponse(
    long? Id,
    bool Ok,
    int HelperPid,
    long SampleIndex,
    string Command,
    IReadOnlyList<HardwareRecord> Hardware,
    IReadOnlyList<string> Errors);

public static class ProtocolJson
{
    public static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
    };
}

public static class SensorUnits
{
    public static string? For(SensorType sensorType) => sensorType switch
    {
        SensorType.Temperature => "C",
        SensorType.Load => "%",
        SensorType.Control => "%",
        SensorType.Clock => "MHz",
        SensorType.Power => "W",
        SensorType.Fan => "RPM",
        SensorType.Voltage => "V",
        SensorType.Data => "GB",
        SensorType.Throughput => "B/s",
        _ => null,
    };
}

public static class SensorValues
{
    public static double? Normalize(
        float? value,
        ICollection<string> errors,
        string identifier)
    {
        if (value is null)
        {
            return null;
        }

        var result = (double)value.Value;
        if (double.IsFinite(result))
        {
            return result;
        }

        errors.Add($"Sensor {identifier} returned a non-finite value.");
        return null;
    }
}
